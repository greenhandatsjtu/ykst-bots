# ykst-bots

[亦可赛艇](https://treehole.space/)的bots们

## 目录结构

+ `ykst_client`：亦可赛艇API封装库 (WIP)
+ `bots/src/bin`：
  + `get-token`：用于获取亦可赛艇token
  + `demo-bot`：示例bot
  + `wordle-bot`：[Wordle](https://www.nytimes.com/games/wordle/index.html) bot

## 使用方法

+ `git clone git@github.com:greenhandatsjtu/ykst-bots.git`

+ `cd ykst-bots`

+ 编译 `cargo build --release`（这一步可能编译失败，可根据提示自行安装缺失包，如`libssl-dev`、`cmake`）

+ 复制配置文件 `cp bots/config.sample.yaml bots/`

+ 编辑`config.yaml`：

  ```yaml
  API_URL: TREEHOLE_API_URL # 亦可赛艇API URL
  TREEHOLE_TOKEN: TREEHOLE_JWT_TOKEN # 亦可赛艇 jwt token，使用get-token.rs获取
  IDENTITY_CODE: YOUR_IDENTITY_CODE # bot所用身份
  THREAD_ID: THREAD_ID # bot所在帖子ID
  
  # for get-token.rs
  AUTH_API_URL: TREEHOLE_AUTH_API_URL # 登录所用的亦可赛艇API URL
  AUTH_REDIRECT_URL: TREEHOLE_AUTH_REDIRECT_API_URL # 亦可赛艇OAuth跳转URL
  ```

+ `cd bots`
+ 浏览器登录jaccount，获取jaccount相关cookies，并设置环境变量：`export JACCOUNT_COOKIE=xxxx`
+ 运行：`../target/release/get-token` 获取亦可赛艇token，粘贴到配置的`TREEHOLE_TOKEN`
+ 后台运行 Wordle bot： `RUST_LOG=info nohup ../target/release/wordle-bot &`
