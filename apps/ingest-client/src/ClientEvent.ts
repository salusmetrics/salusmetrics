export const enum EventType {
  Visitor = 1,
  Session = 2,
  Section = 3,
  Click = 4
}

export interface ClientEvent {
  t: EventType;
  i: string;
  a: Record<string, string> | undefined;
}

export interface ToClientEvent {
  toClientEvent(): ClientEvent;
}
