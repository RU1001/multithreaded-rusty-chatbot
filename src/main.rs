use simple_web_server::ThreadPool;
use rusqlite::{Connection, Result};
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

fn main() {
    // Initialize database
    let conn = init_db().unwrap_or_else(|e| {
        eprintln!("Failed to initialize database: {}", e);
        panic!("Cannot start server without database");
    });
    let db = Arc::new(Mutex::new(conn));

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap_or_else(|e| {
        eprintln!("Failed to bind to port 7878: {}", e);
        panic!("Cannot start server");
    });
    
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        };
        
        let db = Arc::clone(&db);

        pool.execute(move || {
            handle_connection(stream, db);
        });
    }

    println!("Shutting down.");
}

fn init_db() -> Result<Connection> {
    let conn = Connection::open("chat.db")?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS chats (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_message TEXT NOT NULL,
            ai_response TEXT NOT NULL,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;
    
    println!("Database initialized!");
    Ok(conn)
}

fn save_chat(conn: &Connection, user_msg: &str, ai_msg: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO chats (user_message, ai_response) VALUES (?1, ?2)",
        [user_msg, ai_msg],
    )?;
    Ok(())
}

fn get_history(conn: &Connection) -> Result<String> {
    let mut stmt = conn.prepare(
        "SELECT id, user_message, ai_response, timestamp FROM chats ORDER BY timestamp DESC LIMIT 50"
    )?;
    
    let chats = stmt.query_map([], |row| {
        Ok(format!(
            "<div style='margin-bottom: 20px; padding: 15px; background: #f5f5f5; border-radius: 8px;'>
                <div style='color: #666; font-size: 12px; margin-bottom: 5px;'>{}</div>
                <div style='margin-bottom: 10px;'><strong>You:</strong> {}</div>
                <div><strong>Claude:</strong> {}</div>
            </div>",
            row.get::<_, String>(3)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?
        ))
    })?;
    
    let mut html = String::from(
        "<!DOCTYPE html>
        <html>
        <head>
            <title>Chat History</title>
            <style>
                body { font-family: Arial, sans-serif; max-width: 800px; margin: 40px auto; padding: 20px; }
                h1 { color: #667eea; }
                a { color: #667eea; text-decoration: none; }
            </style>
        </head>
        <body>
            <h1>💬 Chat History</h1>
            <a href='/'>← Back to Chat</a>
            <div style='margin-top: 30px;'>"
    );
    
    for chat in chats {
        html.push_str(&chat?);
    }
    
    html.push_str("</div></body></html>");
    Ok(html)
}

fn handle_connection(mut stream: TcpStream, db: Arc<Mutex<Connection>>) {
    let mut buf_reader = BufReader::new(&mut stream);
    
    // Read request line
    let mut request_line = String::new();
    if let Err(e) = buf_reader.read_line(&mut request_line) {
        eprintln!("Failed to read request line: {}", e);
        return;
    }
    let request_line = request_line.trim().to_string();
    
    // Read headers
    let mut content_length = 0;
    loop {
        let mut line = String::new();
        if let Err(e) = buf_reader.read_line(&mut line) {
            eprintln!("Failed to read header: {}", e);
            return;
        }
        if line == "\r\n" || line == "\n" {
            break;
        }
        if line.to_lowercase().starts_with("content-length:") {
            content_length = line.split(':')
                .nth(1)
                .unwrap_or("0")
                .trim()
                .parse()
                .unwrap_or(0);
        }
    }

    let (status_line, response_body) = match request_line.as_str() {
        "GET / HTTP/1.1" => {
            let contents = fs::read_to_string("chat.html")
                .unwrap_or_else(|e| {
                    eprintln!("Failed to read chat.html: {}", e);
                    String::from("<h1>500 - Error loading chat page</h1>")
                });
            ("HTTP/1.1 200 OK", contents)
        }
        "GET /history HTTP/1.1" => {
            let history = match db.lock() {
                Ok(conn) => get_history(&conn).unwrap_or_else(|e| {
                    eprintln!("Failed to load history: {}", e);
                    String::from("<h1>Error loading history</h1>")
                }),
                Err(e) => {
                    eprintln!("Failed to lock database: {}", e);
                    String::from("<h1>Database error</h1>")
                }
            };
            ("HTTP/1.1 200 OK", history)
        }
        "OPTIONS /chat HTTP/1.1" => {
            ("HTTP/1.1 200 OK", String::new())
        }
        "POST /chat HTTP/1.1" => {
            let mut body = vec![0; content_length];
            if let Err(e) = buf_reader.read_exact(&mut body) {
                eprintln!("Failed to read request body: {}", e);
                return;
            }
            
            let body_str = String::from_utf8(body)
                .unwrap_or_else(|e| {
                    eprintln!("Invalid UTF-8 in request: {}", e);
                    String::from("Invalid message")
                });
            
            // Call Claude API
            let ai_response = call_claude_api(&body_str);
            
            // Save to database (don't fail request if save fails)
            if let Ok(conn) = db.lock() {
                if let Err(e) = save_chat(&conn, &body_str, &ai_response) {
                    eprintln!("Failed to save chat to database: {}", e);
                }
            } else {
                eprintln!("Failed to lock database for saving");
            }
            
            ("HTTP/1.1 200 OK", ai_response)
        }
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            let contents = fs::read_to_string("hello.html")
                .unwrap_or_else(|e| {
                    eprintln!("Failed to read hello.html: {}", e);
                    String::from("<h1>Hello World!</h1>")
                });
            ("HTTP/1.1 200 OK", contents)
        }
        _ => {
            let contents = fs::read_to_string("404.html")
                .unwrap_or_else(|e| {
                    eprintln!("Failed to read 404.html: {}", e);
                    String::from("<h1>404 - Not Found</h1>")
                });
            ("HTTP/1.1 404 NOT FOUND", contents)
        }
    };
    
    let length = response_body.len();

    let response = if request_line.contains("/chat") {
        format!(
            "{status_line}\r\nContent-Length: {length}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: POST, GET, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Type: text/plain\r\n\r\n{response_body}"
        )
    } else {
        format!(
            "{status_line}\r\nContent-Length: {length}\r\nContent-Type: text/html\r\n\r\n{response_body}"
        )
    };

    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Failed to write response: {}", e);
    }
}

fn call_claude_api(user_message: &str) -> String {
    let api_key = std::env::var("CLAUDE_API_KEY")
        .expect("CLAUDE_API_KEY environment variable must be set");

    let client = reqwest::blocking::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 1024,
            "messages": [
                {"role": "user", "content": user_message}
            ]
        }))
        .send();

    match response {
        Ok(resp) => {
            match resp.json::<serde_json::Value>() {
                Ok(json) => {
                    json["content"][0]["text"]
                        .as_str()
                        .unwrap_or("Error: Could not parse AI response")
                        .to_string()
                }
                Err(e) => {
                    eprintln!("Failed to parse Claude response: {}", e);
                    String::from("Error: Could not parse AI response")
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to call Claude API: {}", e);
            format!("Error: Could not reach AI service - {}", e)
        }
    }
}
