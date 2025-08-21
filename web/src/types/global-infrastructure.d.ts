// TypeScript definitions for global infrastructure
// Disable strict checking for infrastructure simulation files

/* eslint-disable @typescript-eslint/no-unused-vars */
/* eslint-disable @typescript-eslint/no-explicit-any */

export interface SimpleEventEmitter {
  events: { [event: string]: any[] };
  on(event: string, listener: any): any;
  emit(event: string, ...args: any[]): any;
}

export type EventListener = (...args: any[]) => void;