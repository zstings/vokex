//! 安全相关的 JavaScript 注入代码
//! 
//! 这些代码会在每个 WebView 初始化时执行，用于：
//! - 注入 CSP meta 标签（限制 iframe）
//! - 验证 postMessage 来源
//! - 防止 XSS 攻击

/// 获取安全注入脚本
/// 
/// 包含以下安全措施：
/// 1. CSP meta 标签 - 限制 frame-src
/// 2. iframe 创建拦截 - 阻止远端 iframe
/// 3. postMessage 来源验证 - 防止跨域消息注入
pub fn get_security_script() -> &'static str {
    r#"
(function() {
    'use strict';
    
    // ============================================================
    // 1. CSP meta 标签注入
    // ============================================================
    // 注意：meta 标签的 CSP 能力有限，主要用于提示浏览器限制
    // 真正的 CSP 保护需要 HTTP 头，这里是兜底方案
    (function injectCSP() {
        // 延迟注入，确保 DOM 加载完成
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', injectCSP);
            return;
        }
        
        var meta = document.createElement('meta');
        meta.httpEquiv = 'Content-Security-Policy';
        meta.content = "frame-src 'self' vokex:*; default-src 'self'";
        var head = document.head || document.documentElement;
        if (head) {
            head.insertBefore(meta, head.firstChild);
        }
    })();
    
    // ============================================================
    // 2. iframe 创建拦截
    // ============================================================
    // 阻止通过 document.createElement('iframe') 创建远端 iframe
    (function protectIframe() {
        var _createElement = document.createElement.bind(document);
        document.createElement = function(tagName) {
            var element = _createElement(tagName);
            if (tagName.toLowerCase() === 'iframe') {
                // 拦截 src 属性设置
                Object.defineProperty(element, 'src', {
                    set: function(value) {
                        if (value && (value.startsWith('http://') || value.startsWith('https://'))) {
                            console.warn('[Security] Blocked iframe src:', value);
                            return;
                        }
                        this.setAttribute('src', value);
                    },
                    get: function() {
                        return this.getAttribute('src') || '';
                    }
                });
                // 拦截 srcdoc 属性
                Object.defineProperty(element, 'srcdoc', {
                    set: function(value) {
                        this.setAttribute('srcdoc', value);
                    },
                    get: function() {
                        return this.getAttribute('srcdoc') || '';
                    }
                });
            }
            return element;
        };
    })();
    
    // ============================================================
    // 3. 阻止嵌套 iframe（如果当前在 iframe 中运行）
    // ============================================================
    (function preventNestedFrame() {
        // 检查是否在 iframe 中
        if (window !== window.parent) {
            // 已经是嵌套的，检查父框架是否同源
            try {
                var parentOrigin = window.parent.location.href;
            } catch (e) {
                // 跨域访问会抛出异常，说明不同源
                console.warn('[Security] Blocked cross-origin iframe access');
                document.body.innerHTML = '<div style="display:flex;align-items:center;justify-content:center;height:100vh;font-family:sans-serif;color:#333;"><div style="text-align:center;"><h2>访问被阻止</h2><p>此页面不允许在 iframe 中运行</p></div></div>';
            }
        }
    })();
    
    // ============================================================
    // 4. postMessage 来源验证
    // ============================================================
    // 确保只有来自同源的 postMessage 被处理
    (function protectPostMessage() {
        var _postMessage = window.postMessage.bind(window);
        var _addEventListener = window.addEventListener.bind(window);
        
        // 记录当前页面来源
        var currentOrigin = location.origin || location.protocol + '//' + location.host;
        
        // 重写 postMessage（允许发送，但不做限制）
        // 注意：我们主要验证接收，不限制发送
        
        // 重写 addEventListener 来验证来源
        window.addEventListener = function(type, listener, options) {
            if (type === 'message') {
                var wrappedListener = function(event) {
                    // 验证来源：如果 event.origin 为空（本地页面）或来自可信来源，才处理
                    if (!event.origin || event.origin === '' || event.origin === 'null' || event.origin.startsWith('vokex://') || event.origin.startsWith('http://localhost') || event.origin.startsWith('http://127.0.0.1')) {
                        // 同源或本地消息，安全
                        return listener.call(this, event);
                    }
                    // 来自其他源的 postMessage，记录警告但不处理
                    console.warn('[Security] Blocked postMessage from:', event.origin);
                };
                return _addEventListener.call(this, type, wrappedListener, options);
            }
            return _addEventListener.call(this, type, listener, options);
        };
    })();
    
    // ============================================================
    // 5. 防止 eval 注入（如果 Vokex 允许的话可以扩展）
    // ============================================================
    // 注意：这个功能默认不启用，因为可能影响合法使用
    // 如果需要严格模式，可以取消下面的注释
    /*
    (function restrictEval() {
        var _eval = window.eval;
        window.eval = function(code) {
            console.warn('[Security] eval called, blocking for security');
            throw new Error('eval is disabled for security reasons');
        };
    })();
    */
})();
"#
}
