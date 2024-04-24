import { IApi } from '@umijs/max';

export default (api: IApi) => {
    api.modifyHTML(($) => {
        $('head').append([
            `<script src="//g.alicdn.com/chatui/icons/2.0.2/index.js"></script>`,
        ]);
        return $;
    });
};