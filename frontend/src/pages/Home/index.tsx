import Guide from '@/components/Guide';
import { trim } from '@/utils/format';
import { PageContainer } from '@ant-design/pro-components';
import { useModel } from '@umijs/max';
import styles from './index.less';
import ChatApp from '@/components/Chat';
import { request } from '@@/plugin-request'
import { useEffect, useState } from 'react';

const HomePage: React.FC = () => {

  return (
    <PageContainer ghost>
      <div className={styles.container}>
        <ChatApp />
      </div>
    </PageContainer>
  );
};

export default HomePage;

// 注意：需要稍微修改一下createSession函数，让它能够正确地返回sessionId  
