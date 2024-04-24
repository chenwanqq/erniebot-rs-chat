# erniebot-rs-chat

一个基于[erniebot-rs](https://github.com/chenwanqq/erniebot-rs)库写的对话机器人demo。erniebot-rs是本人写的百度文心大模型的rust SDK。本chat demo是对该SDK的一个综合性的示例。同时在开发本demo的过程中，发现了erniebot-rs项目的若干代码缺陷，erniebot-rs从0.1.1迭代到了0.4.1。

## 技术栈
* 前端：React，Chat Ui，Ant Design, socket.io
* 后端：axum，socketioxide, erniebot-rs，sea-orm

## 实现的功能
* 单步agent，流程为：
    1. 根据用户输入，调用大模型生成函数选择、函数输入
    2. 根据函数选择，生成函数输出
    3. （可选）根据函数输出，调用大模型进行后处理，生成最终输出

目前实现的函数有：
* direct_reply： 直接回复
* calculator：调用evalexpr计算表达式
* document_summary: 生成上传文档的摘要

## 部署流程

### 后端

#### 导入API秘钥
```bash
export QIANFAN_AK=*your_ak*
export QIANFAN_SK=*your_sk*
```

#### 依赖项
```bash
cargo install sea-orm-cli # 操作sea-orm的程序
sudo apt install poppler-utils #解析pdf的工具
```

#### 数据库配置
```bash
docker run --name mysql -v /path/mysql:/var/lib/mysql -e MYSQL_ROOT_PASSWORD=123456 -d -p 3306:3306 mysql:latest # 使用docker启动mysql，然后进入mysql，创建数据库,如本示例中 CREATE DATABASE longtext_demo

cd backend # 在后端的根目录下执行以下命令
DATABASE_URL="mysql://root:123456@localhost:3306/longtext_demo" sea-orm-cli migrate refresh
sea-orm-cli generate entity \ 
    -u mysql://root:123456@localhost:3306/longtext_demo \
    -o src/entities 
``` 

#### 运行
```bash
cargo run
```

### 前端
```bash
cd frontend
pnpm install
pnpm dev
```

