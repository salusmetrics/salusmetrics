import { EventPublishError, EventPublishResult } from "./EventPublisher";
import {
  EventConfiguration,
  EventConfigurationState,
} from "../Event/EventConfiguration";
import { PublishEvent } from "./PublishEvent";

export enum WorkerMessageType {
  ConfigureRequest,
  ConfigureResult,
  PublishError,
  PublishRequest,
  PublishResult,
}

export interface WorkerMessage {
  id: string;
  messageType: WorkerMessageType;
}

export interface WorkerConfigurationRequestMessage extends WorkerMessage {
  messageType: WorkerMessageType.ConfigureRequest;
  config: EventConfiguration;
}

export interface WorkerConfigurationResultMessage extends WorkerMessage {
  messageType: WorkerMessageType.ConfigureResult;
  configurationState: EventConfigurationState;
}

export interface WorkerPublishErrorMessage extends WorkerMessage {
  messageType: WorkerMessageType.PublishError;
  errorType: EventPublishError;
  errorMessage: string;
}

export interface WorkerPublishRequestMessage extends WorkerMessage {
  messageType: WorkerMessageType.PublishRequest;
  events: PublishEvent[];
}

export interface WorkerPublishResultMessage extends WorkerMessage {
  messageType: WorkerMessageType.PublishResult;
  result: EventPublishResult;
}
