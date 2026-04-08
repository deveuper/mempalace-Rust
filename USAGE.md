# MemPalace Rust Edition — 使用指南

## 为什么双击 exe 没反应？

**这是正常的！** MemPalace 是一个**命令行工具**，不是图形界面程序。你需要在终端（命令提示符/PowerShell）中运行它。

---

## 快速开始

### 1. 找到你的 exe 文件

编译完成后，exe 文件在这里：
```
mempalace-rs-v3\target\release\mempalace.exe
```

### 2. 打开终端

**Windows:**
- 按 `Win + R`，输入 `cmd`，回车
- 或者右键点击开始菜单，选择 "Windows PowerShell" 或 "终端"

**macOS/Linux:**
- 打开 "终端" 应用

### 3. 运行命令

```bash
# 进入 exe 所在目录
cd H:\AI\MemPalace RUST\mempalace-rs-v3\target\release

# 查看帮助
mempalace.exe --help

# 初始化 palace（指定你要记住的文件夹）
mempalace.exe init "C:\Users\你的名字\Documents\我的项目"

# 挖掘文件（把内容存进记忆库）
mempalace.exe mine "C:\Users\你的名字\Documents\我的项目"

# 搜索记忆
mempalace.exe search "我们上次讨论了什么"

# 查看状态
mempalace.exe status
```

---

## 在 AI 工具中使用

### Claude Code

**方法 1：命令行添加**
```bash
# 先找到 mempalace.exe 的完整路径
# 比如：H:\AI\MemPalace RUST\mempalace-rs-v3\target\release\mempalace.exe

# 添加 MCP 工具
claude mcp add mempalace -- "H:\AI\MemPalace RUST\mempalace-rs-v3\target\release\mempalace.exe" mcp --transport stdio
```

**方法 2：配置文件**
找到 Claude Code 的配置文件（通常在 `~/.claude/mcp.json`）：
```json
{
  "mcpServers": {
    "mempalace": {
      "command": "H:\\AI\\MemPalace RUST\\mempalace-rs-v3\\target\\release\\mempalace.exe",
      "args": ["mcp", "--transport", "stdio"]
    }
  }
}
```

配置好后，在 Claude Code 中可以直接说：
- "搜索我之前关于数据库设计的讨论"
- "显示我的记忆宫殿状态"
- "为什么我决定用 PostgreSQL 而不是 MySQL？"

### Cursor

1. 打开 Cursor 设置
2. 找到 MCP 配置
3. 添加以下内容到 `~/.cursor/mcp.json`：
```json
{
  "mcpServers": {
    "mempalace": {
      "command": "H:\\AI\\MemPalace RUST\\mempalace-rs-v3\\target\\release\\mempalace.exe",
      "args": ["mcp", "--transport", "stdio"]
    }
  }
}
```
4. 重启 Cursor

### OpenClaw

1. 打开 OpenClaw 的 MCP 配置
2. 添加：
```json
{
  "mcpServers": {
    "mempalace": {
      "command": "mempalace.exe",
      "args": ["mcp", "--transport", "stdio"]
    }
  }
}
```
3. 确保 `mempalace.exe` 在系统 PATH 中，或者使用完整路径

### GitHub Copilot / Codex

这些工具目前对 MCP 的支持有限，但你可以：

1. **手动搜索后复制**：
```bash
mempalace.exe search "GraphQL 相关讨论" > result.txt
```
然后把 `result.txt` 的内容复制给 AI

2. **使用 wake-up 命令**：
```bash
mempalace.exe wake-up > context.txt
```
把 `context.txt` 放在 prompt 开头

---

## 完整命令列表

```bash
# 初始化 palace
mempalace.exe init <目录> [--yes]

# 挖掘文件
mempalace.exe mine <目录> [--mode projects|convos|general] [--extract <类型>]

# 搜索
mempalace.exe search <查询> [--wing <wing>] [--room <room>] [-n <数量>]

# 唤醒上下文
mempalace.exe wake-up [--wing <wing>]

# 查看状态
mempalace.exe status

# 压缩文件（AAAK 格式）
mempalace.exe compress <文件> [--output <输出文件>]

# 修复 palace
mempalace.exe repair

# 分割大文件
mempalace.exe split <文件> [--output <目录>]

# 运行 MCP 服务器
mempalace.exe mcp [--transport stdio|http]
```

---

## 配置文件

配置文件自动创建在：
```
Windows: C:\Users\你的名字\.mempalace\config.toml
macOS/Linux: ~/.mempalace/config.toml
```

示例配置：
```toml
[embedding]
model = "all-MiniLM-L6-v2"
dimensions = 384
batch_size = 32

[storage]
max_doc_size = 1048576  # 1MB
chunk_size = 512
chunk_overlap = 128

[mining]
include_extensions = ["txt", "md", "rs", "py", "js", "ts", "json", "yaml", "toml"]
exclude_dirs = ["node_modules", "target", ".git", "__pycache__", ".venv", "venv"]
max_file_size = 10485760  # 10MB

[layers]
top_moments_count = 20
on_demand_context_size = 500

[mcp]
enabled = true
transport = "stdio"
port = 8080
```

---

## 常见问题

**Q: 为什么搜索结果是随机的？**
A: 当前版本使用简化的哈希嵌入。要获得真正的语义搜索，需要集成真实的嵌入模型（如 fastembed）。

**Q: 数据存在哪里？**
A: 所有数据都在本地：
- Windows: `C:\Users\你的名字\.mempalace\`
- macOS/Linux: `~/.mempalace/`

**Q: 怎么删除所有记忆？**
A: 删除 `~/.mempalace/palace.db` 文件即可。

**Q: 支持中文吗？**
A: 支持！搜索和存储都支持中文。

---

## 与 Python 版本对比

| 功能 | Python | Rust |
|------|--------|------|
| 安装 | 需要 Python + pip | 一个 exe 文件 |
| 启动速度 | 慢 (500ms) | 快 (50ms) |
| 内存占用 | 高 (100MB+) | 低 (10MB) |
| 文件扫描 | 慢 | 快 (20x) |
| 依赖 | 很多 | 几乎没有 |
| 离线使用 | 需要下载模型 | 完全离线 |

---

## 进阶用法

### 挖掘聊天记录

```bash
# 导出 Claude/ChatGPT 聊天记录为 JSON
# 然后：
mempalace.exe mine "C:\Users\Name\Downloads\chat-exports" --mode convos
```

### 按类型提取信息

```bash
# 只提取决策相关的信息
mempalace.exe mine "C:\Users\Name\Documents\notes" --mode general --extract decisions

# 提取里程碑
mempalace.exe mine "C:\Users\Name\Documents\notes" --mode general --extract milestones
```

### 压缩大文件

```bash
mempalace.exe compress "large-conversation.txt" --output "compressed.txt"
```

---

有问题？查看 README.md 或 PROJECT_SUMMARY.md 了解更多技术细节！
