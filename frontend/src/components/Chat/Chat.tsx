import { useEffect, useState } from 'react';
import '@chatui/core/es/styles/index.less';
import '@chatui/core/dist/index.css';
import Chat, { Bubble, useMessages } from '@chatui/core';
import { Modal, Upload, Button, message } from 'antd';
import { UploadOutlined } from '@ant-design/icons';
import { request } from '@@/plugin-request'
import { RcFile } from 'antd/lib/upload';
import {io,Socket} from 'socket.io-client';

const initialMessages = [
  {
    type: 'text',
    content: { text: '主人好，我是智能助理，可以使用多种工具~' },
    user: { avatar: '//gw.alicdn.com/tfs/TB1DYHLwMHqK1RjSZFEXXcGMXXa-56-62.svg' },
  },
];

// 默认快捷短语，可选
const defaultQuickReplies = [
  {
    name: '短语1',
    isNew: true,
  },
  {
    name: '短语2',
    isHighlight: true,
  },
  {
    name: '短语3',
  },
];

const ChatApp = function () {
  // 消息列表
  const { messages, appendMsg, setTyping } = useMessages(initialMessages);
  const [isModalVisible, setIsModalVisible] = useState(false);
  const [sessionId, setSessionId] = useState(-1); // 初始化sessionId状态  
  const [socket, setSocket] = useState<Socket | null>(null);



  const createSession = async () => {
    try {
      const res = await request('/create_session', {
        'method': 'POST',
        'data': {
          'user_id': 1234
        }
      });
      if (res.code === 200) {
        return res.data.session_id; // 直接返回sessionId，而不是在then里面处理  
      } else {
        console.log("error!")
        console.log(res);
      }
    } catch (error) {
      console.log(error);
    }
  }

  useEffect(() => {
    // 组件挂载后调用createSession  
    createSession().then(newSessionId => {
      console.log("newSessionId: " + newSessionId);
      if (newSessionId !== undefined) {
        setSessionId(newSessionId); // 更新sessionId状态 
      }
    });
    const socket = io('http://localhost:8888/ws', { transports: ['websocket', 'polling', 'flashsocket'] });
    setSocket(socket);
    socket.connect();
    function onResponse(res: any) {
      console.log("start response");
      if (res.code === 200) {
        appendMsg({
          type: 'text',
          content: { text: res.data.response },
          position: 'left',
        });
      } else {
        console.log('error!');
        console.log(res);
      }
      setTyping(false);
    }
    function onConnect() {
      console.log("connected");
    }

    function onDisconnect() {
      console.log("disconnected");
    }
    // 监听socket.io事件
    socket.on("response", onResponse);
    socket.on("connect", onConnect);
    socket.on("disconnect", onDisconnect);
    return () => {
      socket.disconnect();
      socket.off("response", onResponse);
      socket.off("connect", onConnect);
      socket.off("disconnect", onDisconnect);
    }
  }, []); // 空依赖数组表示这个effect只会在组件挂载时运行一次  

  const toolbar = [
    {
      type: 'file',
      icon: 'file',
      title: '文件',
    }
  ];
  // 发送回调
  async function handleSend(type: string, val: string) {
    if (type === 'text' && val.trim()) {
      // TODO: 发送请求
      appendMsg({
        type: 'text',
        content: { text: val },
        position: 'right',
      });

      setTyping(true);
      socket.emit("chat", {
        content: val,
        content_type: 'text',
        session_id: sessionId
      });
      /*
      request('/reply_chat',
        {
          method: 'POST',
          data: {
            content: val,
            content_type: 'text',
            session_id: sessionId
          }
        }
      ).then(
        (res: any) => {
          if (res.code === 200) {
            appendMsg({
              type: 'text',
              content: { text: res.data.response },
              position: 'left',
            });
          } else {
            console.log('error!');
            console.log(res);
          }
          setTyping(false);
        }
      ).catch(
        (error: any) => {
          console.log(error)
          setTyping(false);
        }
      );
      */
    }
  }

  // 快捷短语回调，可根据 item 数据做出不同的操作，这里以发送文本消息为例
  function handleQuickReplyClick(item: any) {
    handleSend('text', item.name);
  }

  function renderMessageContent(msg: any) {
    const { type, content } = msg;

    // 根据消息类型来渲染
    switch (type) {
      case 'text':
        return <Bubble content={content.text} />;
      case 'image':
        return (
          <Bubble type="image">
            <img src={content.picUrl} alt="" />
          </Bubble>
        );
      default:
        return null;
    }
  }

  function handleToolBarClick(item: any) {
    if (item.type === 'file') {
      setIsModalVisible(true);
    }
  }

  const uploadData = {
    sessionId: sessionId,
  }

  // 文件上传前的处理  
  const beforeUpload = (file: RcFile) => {
    // 这里可以进行文件验证等操作  
    console.log('Before upload:', file);
    return true;
  };

  const onChange = (info: any) => {
    if (info.file.status === 'uploading') {
      message.info("文件上传中...");
    } else if (info.file.status === 'done') {
      setIsModalVisible(false);
      if (info.file.response.code === 200) {
        message.success("文件上传成功");
        appendMsg({
          type: 'text',
          content: { text: '您已经上传了' + info.file.name + '你可以提问有关这篇文档的问题了！' },
          position: 'left',
        });
      } else {
        message.error("文件上传失败");
      }
    }
  }


  return (
    <div>
      <Modal
        title="文件上传"
        open={isModalVisible}
        onCancel={() => setIsModalVisible(false)}
        footer={null}
      >
        <Upload
          action={'http://localhost:8888/upload'}
          beforeUpload={beforeUpload}
          showUploadList={false}
          data={uploadData}
          onChange={onChange}
        >
          <Button icon={<UploadOutlined />}>选择文件</Button>
        </Upload>
      </Modal>
      {sessionId != -1 && <Chat
        navbar={{ title: '智能助理' }}
        messages={messages}
        renderMessageContent={renderMessageContent}
        quickReplies={defaultQuickReplies}
        onQuickReplyClick={handleQuickReplyClick}
        onSend={handleSend}
        toolbar={toolbar}
        onToolbarClick={handleToolBarClick}
      />}
    </div>
  )
}
export default ChatApp;
