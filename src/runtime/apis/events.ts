/**
 * 事件监听相关 API
 */
export const events = {
  /**
   * 监听事件
   */
  on: (event: string, listener: (data: any) => void): (() => void) => {
    const vokex = (window as any).__VOKEX__;
    if (vokex?.on) {
      vokex.on(event, listener);
      return () => vokex.off(event, listener);
    }
    return () => {};
  },

  /**
   * 取消监听事件
   */
  off: (event: string, listener: (data: any) => void): void => {
    const vokex = (window as any).__VOKEX__;
    if (vokex?.off) {
      vokex.off(event, listener);
    }
  },

  /**
   * 触发事件（内部使用）
   */
  emit: (event: string, data?: any): void => {
    const vokex = (window as any).__VOKEX__;
    if (vokex?.__emit__) {
      vokex.__emit__(event, data);
    }
  },
};