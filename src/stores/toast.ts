import { defineStore } from "pinia";

export enum ToastType {
  INFO = "info",
  SUCCESS = "success",
  WARNING = "warning",
  ERROR = "error",
}

export interface Toast {
  id: string;
  message: string;
  type: ToastType;
  duration?: number;
}

export const useToastStore = defineStore("toast", {
  state: (): {
    toasts: Toast[];
  } => ({
    toasts: [],
  }),
  actions: {
    addToast(message: string, type: ToastType = ToastType.INFO, duration = 10_000) {
      const id = crypto.randomUUID();
      const toast: Toast = { id, message, type, duration };
      this.toasts.push(toast);

      if (duration > 0) {
        setTimeout(() => {
          this.removeToast(id);
        }, duration);
      }
    },

    removeToast(id: string) {
      const index = this.toasts.findIndex((t) => t.id === id);
      if (index !== -1) {
        this.toasts.splice(index, 1);
      }
    },
  },
});
