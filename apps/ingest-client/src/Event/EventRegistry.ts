import { EventReference } from "./Event";

export const enum RegisterEventError {
  BadRequestError,
  ConfigurationError,
  InternalError,
}

export type RegisterEventResult = RegisterEventError | void;

export interface SyncEventRegistry {
  registerVisitor(): RegisterEventResult;
  deregisterVisitor(): RegisterEventResult;
  registerSession(): RegisterEventResult;
  deregisterSession(): RegisterEventResult;
  registerSection(): RegisterEventResult;
  deregisterSection(): RegisterEventResult;
  registerClick(): RegisterEventResult;
}

export const enum EventRegistryError {
  BadRequestError,
  ConfigurationError,
  InternalError,
}

export type EventRegistryResult = EventReference | EventRegistryError;

export interface EventRegistry {
  registerVisitor(): Promise<EventRegistryResult>;
  deregisterVisitor(): Promise<EventRegistryResult>;
  registerSession(): Promise<EventRegistryResult>;
  deregisterSession(): Promise<EventRegistryResult>;
  registerSection(): Promise<EventRegistryResult>;
  deregisterSection(): Promise<EventRegistryResult>;
  registerClick(): Promise<EventRegistryResult>;
}
