export const enum RegisterEventError {
  BadRequestError,
  ConfigurationError,
  InternalError,
}

export type RegisterEventResult = RegisterEventError | void;

export interface EventRegistry {
  registerVisitor(): RegisterEventResult;
  deregisterVisitor(): RegisterEventResult;
  registerSession(): RegisterEventResult;
  deregisterSession(): RegisterEventResult;
  registerSection(): RegisterEventResult;
  deregisterSection(): RegisterEventResult;
  registerClick(): RegisterEventResult;
}
