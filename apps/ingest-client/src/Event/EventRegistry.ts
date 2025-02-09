export const enum RegisterEventError {
  BadRequestError,
  ConfigurationError,
  InternalError,
}

export type RegisterEventResult = RegisterEventError | void;

export interface EventRegistrySuccess {
  eventCount: number;
}

export const enum EventRegistryError {
  BadRequest,
  ConfigurationError,
  InternalError,
}

export type EventRegistryResult = EventRegistrySuccess | EventRegistryError;

export interface EventRegistry {
  registerVisitor(): Promise<EventRegistryResult>;
  deregisterVisitor(): Promise<EventRegistryResult>;
  registerSession(): Promise<EventRegistryResult>;
  deregisterSession(): Promise<EventRegistryResult>;
  registerSection(): Promise<EventRegistryResult>;
  deregisterSection(): Promise<EventRegistryResult>;
  registerClick(): Promise<EventRegistryResult>;
}
