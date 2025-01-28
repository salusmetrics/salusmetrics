import { Click, Section, Session, Visitor } from './Events';
import { HttpEventPublisher } from './HttpEventPublisher';

let visitor = new Visitor();
let session = new Session(visitor);
let section = new Section(session);
let click = new Click(section);

let publisher = new HttpEventPublisher('abc-xyz', 'http://localhost:3000');

publisher.publish([visitor, session, section, click]);
