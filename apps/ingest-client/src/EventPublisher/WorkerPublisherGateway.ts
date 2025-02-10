import { EventPublisher, EventPublishResult } from "./EventPublisher";
import {
  EventConfiguration,
  EventConfigurationState,
  validateEventConfiguration,
} from "../Event/EventConfiguration";
import { PublishEvent } from "./PublishEvent";
import {
  WorkerConfigurationRequestMessage,
  WorkerConfigurationResultMessage,
  WorkerMessageType,
  WorkerPublishErrorMessage,
  WorkerPublishRequestMessage,
  WorkerPublishResultMessage,
} from "./WorkerMessage";
import { v7 as uuidv7 } from "uuid";

type ResolveEventPublishResult = (
  value: EventPublishResult | PromiseLike<EventPublishResult>,
) => void;
type RejectEventPublishResult = (reason?: any) => void;

interface PromiseHandlers {
  resolve: ResolveEventPublishResult;
  reject: RejectEventPublishResult;
}

export type GlobalPublishErrorHandler = (err?: string) => void;

export class WorkerPublisherGateway implements EventPublisher {
  private workerPublisher: Worker;
  private promiseRegistry: Record<string, PromiseHandlers>;
  private configurationState: EventConfigurationState | undefined;
  private errorHandler: GlobalPublishErrorHandler | undefined;

  constructor(
    config: EventConfiguration,
    errorHandler?: GlobalPublishErrorHandler,
  ) {
    this.errorHandler = errorHandler;
    this.configurationState = undefined;
    if (validateEventConfiguration(config) != EventConfigurationState.Success) {
      if (this.errorHandler != undefined) {
        this.errorHandler(
          "Error constructing WorkerPublisherGateway - Invalid Configuration",
        );
      }
      throw new Error("Invalid Event Publisher Configuration");
    }
    this.promiseRegistry = {};
    this.workerPublisher = new Worker(
      new URL("WorkerPublisher.js", import.meta.url),
    );
    this.workerPublisher.addEventListener("message", (m) =>
      this.routeMessage(m),
    );
    this.workerPublisher.addEventListener("error", (e) => {
      if (this.errorHandler != undefined) {
        this.errorHandler(e.message);
      }
    });
    this.postConfigRequest(config);
  }

  publish(events: PublishEvent[]): Promise<EventPublishResult> {
    const message = this.createPublishMessage(events);
    return new Promise<EventPublishResult>((resolve, reject) => {
      this.registerPublishPromise(message, { resolve, reject });
      this.postPublishRequest(message);
    });
  }

  private routeMessage(message: MessageEvent) {
    const workerMessage = message.data as
      | WorkerConfigurationResultMessage
      | WorkerPublishResultMessage
      | WorkerPublishErrorMessage;

    if (workerMessage.messageType == WorkerMessageType.ConfigureResult) {
      this.configurationState = workerMessage.configurationState;
      if (
        this.configurationState != EventConfigurationState.Success &&
        this.errorHandler != undefined
      ) {
        this.errorHandler("Worker event publisher failed");
      }
    } else {
      this.resolvePublish(workerMessage);
    }
  }

  private resolvePublish(
    resultMessage: WorkerPublishResultMessage | WorkerPublishErrorMessage,
  ) {
    const handlers = this.promiseRegistry[resultMessage.id];
    if (handlers == undefined) {
      if (this.errorHandler != undefined) {
        this.errorHandler(
          "Event Publish Result had no matching handlers in registry",
        );
        return;
      }
    } else {
      if (resultMessage.messageType == WorkerMessageType.PublishResult) {
        handlers.resolve(resultMessage.result);
      } else {
        handlers.reject(resultMessage.errorMessage);
      }
      delete this.promiseRegistry[resultMessage.id];
    }
  }

  private createPublishMessage(
    events: PublishEvent[],
  ): WorkerPublishRequestMessage {
    const id = uuidv7();
    return {
      id,
      messageType: WorkerMessageType.PublishRequest,
      events: events,
    };
  }

  private registerPublishPromise(
    message: WorkerPublishRequestMessage,
    handlers: PromiseHandlers,
  ): void {
    this.promiseRegistry[message.id] = handlers;
  }

  private postPublishRequest(
    publishMessage: WorkerPublishRequestMessage,
  ): void {
    this.workerPublisher.postMessage(publishMessage);
  }

  private postConfigRequest(config: EventConfiguration): void {
    const configMessage: WorkerConfigurationRequestMessage = {
      config,
      id: uuidv7(),
      messageType: WorkerMessageType.ConfigureRequest,
    };
    this.workerPublisher.postMessage(configMessage);
  }
}
