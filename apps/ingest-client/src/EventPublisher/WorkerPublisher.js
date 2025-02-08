import { WorkerPublisherProxy } from "./WorkerPublisherProxy";

/**@type {DedicatedWorkerGlobalScope} */
const ctx = self;
const publisherProxy = new WorkerPublisherProxy(ctx);
ctx.addEventListener("message", (e) => publisherProxy.proxyMessage(e));
