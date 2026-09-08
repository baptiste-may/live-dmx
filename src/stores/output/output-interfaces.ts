import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

import { ToastType, useToastStore } from "@/stores/toast.ts";
import type { PhysicalInterface } from "@/types/output.ts";

export const DEFAULT_PHYSICAL_INTERFACES: PhysicalInterface[] = [
  {
    id: "physical-1",
    name: "Physical 1",
    device: "Enttec DMX USB Pro",
    port: "/dev/ttyUSB0",
    status: "connected",
    fps: 44,
  },
  {
    id: "physical-2",
    name: "Physical 2",
    device: "FTDI Open DMX",
    port: "/dev/ttyUSB1",
    status: "connected",
    fps: 40,
  },
  {
    id: "physical-3",
    name: "Physical 3",
    device: "DMXking ultraDMX MAX",
    port: "/dev/ttyUSB2",
    status: "connected",
    fps: 44,
  },
];

export const useOutputInterfaces = defineStore("output-interfaces", {
  state: () => ({
    interfaces: [] as PhysicalInterface[],
    isScanning: false,
  }),
  actions: {
    async fetchInterfaces() {
      try {
        const result = await invoke<PhysicalInterface[]>("get_output_interfaces");
        if (Array.isArray(result)) {
          this.interfaces = result;
        } else {
          this.interfaces = [];
        }
      } catch {
        this.interfaces = [];
      }
    },
    async scanInterfaces() {
      this.isScanning = true;
      try {
        const result = await invoke<PhysicalInterface[]>("scan_output_interfaces");
        if (Array.isArray(result)) {
          this.interfaces = result;
        }
        useToastStore().addToast(
          `Scanned interfaces: ${this.interfaces.length} found`,
          ToastType.INFO,
        );
      } catch (err: unknown) {
        useToastStore().addToast(`Failed to scan interfaces: ${String(err)}`, ToastType.ERROR);
      } finally {
        this.isScanning = false;
      }
    },
  },
});
