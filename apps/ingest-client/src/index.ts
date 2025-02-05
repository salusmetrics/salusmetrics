import { EventManager } from "./EventManager/EventManager";
import { HttpEventPublisher } from "./EventPublisher/HttpEventPublisher";
import { WebStorageSiteStateRepository } from "./SiteState/WebStorageSiteStateRepository";

let api_key = "abc-xyz";
let siteStateRepository = new WebStorageSiteStateRepository(api_key);
let publisher = new HttpEventPublisher(api_key, "http://localhost:3000");
let eventManager = new EventManager(publisher, siteStateRepository);

eventManager.registerSection();
