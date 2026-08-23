import { describe, it, expect } from "vitest";
import { historyKey, reducer, initial } from "./webphoneStore";
describe("historyKey", () => {
  it("scopes by domain+user", () => {
    expect(historyKey("d1","u1")).toBe("webphone:history:d1:u1");
  });
});
describe("reducer HISTORY_APPEND", () => {
  it("appends and caps 100", () => {
    let s = initial;
    for(let i=0;i<101;i++) s = reducer(s, {type:"HISTORY_APPEND", rec:{id:String(i), direction:"out", peer:"1001", peerName:"A", startedAt:new Date().toISOString(), duration:5, disposition:"answered", domain_id:"d", user_uuid:"u"}});
    expect(s.history.length).toBe(100);
    expect(s.history[0].id).toBe("1");
  });
});
