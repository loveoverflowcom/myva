import init from './shell/myva_web_shell.js';
try {
    await init();
    document.querySelector('#boot-status').remove();
} catch (error) {
    document.querySelector('#boot-status').textContent = 'Không thể tải MyVa. Kiểm tra kết nối và tải lại trang.';
    console.error(error);
}
