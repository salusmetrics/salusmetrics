import { WorkerPublisherProxy } from "./WorkerPublisherProxy";

const ctx = self;
const publisherProxy = new WorkerPublisherProxy(ctx);
ctx.addEventListener("message", (e) => publisherProxy.proxyMessage(e));
