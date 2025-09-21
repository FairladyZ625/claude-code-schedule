# Claude Code Schedule - 后台运行配置指南

## 方案一：使用 systemd 服务（推荐）

### 1. 编译项目
```bash
cargo build --release
```

### 2. 创建日志目录
```bash
sudo mkdir -p /var/log/claude-code-schedule
sudo chown $USER:$USER /var/log/claude-code-schedule
```

### 3. 安装 systemd 服务
```bash
# 复制服务文件
sudo cp claude-code-schedule.service /etc/systemd/system/

# 重新加载 systemd
sudo systemctl daemon-reload

# 启用服务（开机自启）
sudo systemctl enable claude-code-schedule

# 启动服务
sudo systemctl start claude-code-schedule
```

### 4. 管理服务
```bash
# 查看服务状态
sudo systemctl status claude-code-schedule

# 查看日志
sudo journalctl -u claude-code-schedule -f

# 停止服务
sudo systemctl stop claude-code-schedule

# 重启服务
sudo systemctl restart claude-code-schedule

# 禁用服务
sudo systemctl disable claude-code-schedule
```

## 方案二：使用 nohup 命令

### 1. 编译项目
```bash
cargo build --release
```

### 2. 后台运行
```bash
# 创建日志目录
mkdir -p logs

# 使用 nohup 后台运行
nohup ./target/release/ccschedule --loop --ping-mode --log-dir logs --pid-file ccschedule.pid > ccschedule.out 2>&1 &

# 查看进程
ps aux | grep ccschedule

# 停止进程
kill $(cat ccschedule.pid)
```

## 方案三：使用 screen 或 tmux

### 使用 screen
```bash
# 创建新的 screen 会话
screen -S claude-schedule

# 在 screen 中运行程序
./target/release/ccschedule --loop --ping-mode

# 分离会话（Ctrl+A, 然后按 D）
# 重新连接会话
screen -r claude-schedule
```

### 使用 tmux
```bash
# 创建新的 tmux 会话
tmux new-session -d -s claude-schedule

# 在 tmux 中运行程序
tmux send-keys -t claude-schedule './target/release/ccschedule --loop --ping-mode' Enter

# 查看会话
tmux list-sessions

# 连接到会话
tmux attach-session -t claude-schedule
```

## 日志管理

### 日志轮转配置
创建 `/etc/logrotate.d/claude-code-schedule`：
```
/var/log/claude-code-schedule/*.log {
    daily
    missingok
    rotate 30
    compress
    delaycompress
    notifempty
    create 644 root root
}
```

### 监控日志
```bash
# 实时查看日志
tail -f /var/log/claude-code-schedule/$(date +%Y-%m-%d).log

# 查看最近的日志
ls -la /var/log/claude-code-schedule/
```

## 故障排除

### 检查服务状态
```bash
sudo systemctl status claude-code-schedule
sudo journalctl -u claude-code-schedule --since "1 hour ago"
```

### 检查权限
```bash
# 确保可执行文件有执行权限
chmod +x target/release/ccschedule

# 确保日志目录可写
ls -la /var/log/claude-code-schedule/
```

### 手动测试
```bash
# 测试单次执行
./target/release/ccschedule --dry-run --ping-mode

# 测试循环模式
./target/release/ccschedule --dry-run --loop --ping-mode
```

## 安全建议

1. **使用专用用户**：创建专用的系统用户运行服务
2. **限制权限**：确保服务只有必要的文件访问权限
3. **监控日志**：定期检查日志文件，监控异常情况
4. **备份配置**：定期备份服务配置和日志文件
