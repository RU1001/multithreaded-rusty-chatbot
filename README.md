# 🤖 Rust AI Chat Server
![Screencast](screencast.gif)





![Screenshot](screenshot.png)



A multithreaded web server built from scratch in Rust featuring real-time AI conversations with Claude and persistent chat history. This is an extension of the last project in the Rust Programming Language Book.

## ✨ Features

- **Custom ThreadPool Implementation** - 4 worker threads handling concurrent requests
- **AI-Powered Chat** - Integration with Anthropic's Claude API
- **Persistent Storage** - SQLite database stores all conversations
## 🛠️ Tech Stack

- **Rust** - Systems programming language
- **Reqwest** - HTTP client for API calls
- **SQLite (rusqlite)** - Embedded database
- **Claude API** - Anthropic's AI assistant
- **HTML/CSS/JavaScript** - Frontend interface

## 📋 Prerequisites

Before running this project, make sure you have:

1. **Rust installed** (1.70.0 or newer)
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Verify installation
   rustc --version
   cargo --version
   ```

2. **Claude API Key** from [Anthropic Console](https://console.anthropic.com)
   - Sign up for an account
   - Navigate to API Keys section
   - Generate a new API key

3. **OpenSSL development libraries** (Linux only)
   ```bash
   # Ubuntu/Debian
   sudo apt-get install libssl-dev pkg-config
   
   # Fedora
   sudo dnf install openssl-devel
   ```

## 🚀 Installation & Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/rust-ai-chat-server.git
   cd rust-ai-chat-server
   ```

2. **Set your Claude API key**
   
   **Option A: Environment Variable (Recommended)**
   ```bash
   # Linux/Mac
   export CLAUDE_API_KEY=your-api-key-here
   
   # Windows (PowerShell)
   $env:CLAUDE_API_KEY="your-api-key-here"
   
   # Windows (CMD)
   set CLAUDE_API_KEY=your-api-key-here
   ```
   
   **Option B: Temporary for one session**
   ```bash
   CLAUDE_API_KEY=your-key cargo run
   ```

3. **Build and run the server**
   ```bash
   cargo run
   ```

4. **Open your browser**
   
   Navigate to: `http://127.0.0.1:7878`

## 💬 Usage

### Chat Interface
1. Open `http://127.0.0.1:7878` in your browser
2. Type your message in the input box
3. Press Enter or click "Send"
4. View AI responses in real-time

### View Chat History
- Click "📜 View History" button in the top-right corner
- Or navigate to `http://127.0.0.1:7878/history`
- See all past conversations with timestamps

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Main chat interface |
| `/chat` | POST | Send message to AI (returns response) |
| `/history` | GET | View all saved conversations |
| `/sleep` | GET | Demo endpoint (5 second delay) |

## 🧪 Testing the Server

### Test Multithreading
1. Open 3-4 browser tabs to `http://127.0.0.1:7878`
2. Send messages from each tab simultaneously
3. Check terminal - you'll see different workers handling requests:
   ```
   Worker 0 got a job; executing.
   Worker 2 got a job; executing.
   Worker 1 got a job; executing.
   ```

### Test with curl
```bash
# Send a chat message
curl -X POST http://127.0.0.1:7878/chat -d "Hello, who are you?"

# View history (returns HTML)
curl http://127.0.0.1:7878/history
```

## 📁 Project Structure

```
rust-ai-chat-server/
├── src/
│   ├── main.rs          # Server logic, routes, API integration
│   └── lib.rs           # ThreadPool and Worker implementation
├── chat.html            # Chat UI interface
├── hello.html           # Demo page
├── 404.html             # Error page
├── Cargo.toml           # Dependencies
├── chat.db              # SQLite database (created on first run)
└── README.md
```

## 🔧 Configuration

### Change Port
Edit `main.rs` line 13:
```rust
let listener = TcpListener::bind("127.0.0.1:7878").unwrap_or_else(|e| {
```
Change `7878` to your preferred port.

### Adjust Thread Pool Size
Edit `main.rs` line 17:
```rust
let pool = ThreadPool::new(4);
```
Change `4` to your desired number of worker threads.

### Modify AI Model
Edit `call_claude_api` function in `main.rs`:
```rust
"model": "claude-sonnet-4-20250514",  // Change model here
"max_tokens": 1024,                   // Adjust response length
```

## 🐛 Troubleshooting

### "CLAUDE_API_KEY must be set"
**Solution:** Set your API key as an environment variable before running:
```bash
export CLAUDE_API_KEY=your-key-here
cargo run
```

### "Failed to bind to port 7878"
**Solution:** Port is already in use. Either:
- Kill the process using the port
- Change to a different port in `main.rs`

### "libssl-dev not found" (Linux)
**Solution:** Install OpenSSL development libraries:
```bash
sudo apt-get install libssl-dev pkg-config
```

### Chat not working in browser but curl works
**Solution:** Check browser console (F12) for CORS errors. Make sure you're accessing via `http://127.0.0.1:7878` not `localhost`.

## 🎓 Learning Resources

This project demonstrates concepts from:
- [The Rust Programming Language Book](https://doc.rust-lang.org/book/) - Chapter 20: Multithreaded Web Server
- Anthropic Claude API documentation
- Rust async vs sync I/O patterns
- Thread-safe shared state with `Arc<Mutex<T>>`


- Built following [The Rust Book](https://doc.rust-lang.org/book/)
- Powered by [Anthropic's Claude API](https://www.anthropic.com)

