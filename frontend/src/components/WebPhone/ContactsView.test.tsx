import "@testing-library/jest-dom/vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { ContactsView } from "./ContactsView";
describe("ContactsView", ()=>{
  it("calls dial", ()=>{
    const onDial=vi.fn(); render(<ContactsView contacts={[{id:"1", primary:"1001", name:"Alice", favorite:false}]} onDial={onDial} onToggleFav={()=>{}}/>);
    fireEvent.click(screen.getByTestId("contact-dial-1001")); expect(onDial).toHaveBeenCalledWith("1001");
  });
  it("filters by search", ()=>{
    render(<ContactsView contacts={[{id:"1", primary:"1001", name:"Alice", favorite:false},{id:"2", primary:"1002", name:"Bob", favorite:false}]} onDial={()=>{}} onToggleFav={()=>{}}/>);
    fireEvent.change(screen.getByTestId("contacts-search"), {target:{value:"Bob"}});
    expect(screen.queryByText(/1001/)).not.toBeInTheDocument();
    expect(screen.getByText(/1002/)).toBeInTheDocument();
  });
  it("toggles favorite", ()=>{
    const onToggle=vi.fn(); render(<ContactsView contacts={[{id:"1", primary:"1001", name:"Alice", favorite:false}]} onDial={()=>{}} onToggleFav={onToggle}/>);
    fireEvent.click(screen.getByTestId("favorite-1")); expect(onToggle).toHaveBeenCalledWith("1");
  });
});
