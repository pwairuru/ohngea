import * as React from "react";
import { apiFetch } from "../lib/api";
import { Card, CardContent, CardHeader, CardTitle } from "../components/ui/card";
import { Button } from "../components/ui/button";
import { Input } from "../components/ui/input";
import { Label } from "../components/ui/label";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "../components/ui/table";
import { Dialog, DialogHeader, DialogTitle } from "../components/ui/dialog";
import { Badge } from "../components/ui/badge";
import { Pencil, Trash2, Users as UsersIcon, Shield, Plus, Copy, Key, Search } from "lucide-react";

export function Users() {
  const [me, setMe] = React.useState<any>(null);
  const [domains, setDomains] = React.useState<any[]>([]);
  const [permsAll, setPermsAll] = React.useState<any[]>([]);

  // users state like users.php:145 paging 50
  const [users, setUsers] = React.useState<any[]>([]);
  const [usersTotal, setUsersTotal] = React.useState(0);
  const [usersPage, setUsersPage] = React.useState(1);
  const [usersSearch, setUsersSearch] = React.useState("");
  const [usersShowAll, setUsersShowAll] = React.useState(false);
  const [usersOrderBy, setUsersOrderBy] = React.useState("username");
  const [usersOrder, setUsersOrder] = React.useState<"asc"|"desc">("asc");
  const [usersSelected, setUsersSelected] = React.useState<Set<string>>(new Set());

  // groups state like groups.php
  const [groups, setGroups] = React.useState<any[]>([]);
  const [groupsTotal, setGroupsTotal] = React.useState(0);
  const [groupsPage, setGroupsPage] = React.useState(1);
  const [groupsSearch, setGroupsSearch] = React.useState("");
  const [groupsShowAll, setGroupsShowAll] = React.useState(false);
  const [groupsOrderBy, setGroupsOrderBy] = React.useState("group_name");
  const [groupsOrder, setGroupsOrder] = React.useState<"asc"|"desc">("asc");
  const [groupsSelected, setGroupsSelected] = React.useState<Set<string>>(new Set());

  const perPage = 50;

  // dialogs
  const [userOpen, setUserOpen] = React.useState(false);
  const [editingUser, setEditingUser] = React.useState<any | null>(null);
  const [userForm, setUserForm] = React.useState({ username:"", email:"", password:"", password_confirm:"", enabled:true, user_status:"", user_type:"default", contact_uuid:"", user_language:"", user_time_zone:"", api_key:"", user_totp_secret:"", group_uuid_name:"", domain_id:"" });
  // multi-group checkbox state: replaces single group_uuid_name select
  const [selectedGroups, setSelectedGroups] = React.useState<Set<string>>(new Set());
  // delegation helpers for group_level filtering
  const [myLevel, setMyLevel] = React.useState<number>(80);
  const [myGroups, setMyGroups] = React.useState<Set<string>>(new Set());
  // contacts dropdown fetched on open
  const [contacts, setContacts] = React.useState<any[]>([]);
  const [contactSearch, setContactSearch] = React.useState("");
  // api_key view/hide toggle, not plaintext
  const [showApiKey, setShowApiKey] = React.useState(false);

  const [groupOpen, setGroupOpen] = React.useState(false);
  const [editingGroup, setEditingGroup] = React.useState<any | null>(null);
  const [groupForm, setGroupForm] = React.useState({ name:"", description:"", group_level:30, group_protected:false, domain_id:"" });

  const [membersOpen, setMembersOpen] = React.useState(false);
  const [membersGroup, setMembersGroup] = React.useState<any | null>(null);
  const [members, setMembers] = React.useState<any[]>([]);
  const [addMemberId, setAddMemberId] = React.useState("");
  const [membersSelected, setMembersSelected] = React.useState<Set<string>>(new Set());

  const [permsOpen, setPermsOpen] = React.useState(false);
  const [permsGroup, setPermsGroup] = React.useState<any | null>(null);
  const [permsList, setPermsList] = React.useState<any[]>([]);
  const [permsSearch, setPermsSearch] = React.useState("");
  const [permsView, setPermsView] = React.useState("");

  // password strength: length 12 + number/lower/upper/special (defaults 12/true/true/true/false per app_config:119) — preserve Task 3 meter 36-54
  const validatePassword = (pw: string) => {
    const errs: string[] = [];
    if (pw.length < 12) errs.push("at least 12 characters");
    if (!/\d/.test(pw)) errs.push("one number");
    if (!/[a-z]/.test(pw)) errs.push("one lowercase");
    if (!/[A-Z]/.test(pw)) errs.push("one uppercase");
    return errs;
  };
  const pwErrors = React.useMemo(() => {
    if (!userForm.password) return [];
    return validatePassword(userForm.password);
  }, [userForm.password]);
  const isCreate = !editingUser;
  const passwordRequired = isCreate && !userForm.password;
  const isPasswordWeak = isCreate ? (passwordRequired || pwErrors.length > 0) : (userForm.password ? pwErrors.length > 0 : false);
  const passwordScore = userForm.password ? 4 - pwErrors.length : 0;

  const loadMe = React.useCallback(()=>{ apiFetch("/auth/me").then(setMe).catch(()=>{}); },[]);
  const loadDomains = React.useCallback(()=>{ apiFetch("/domains").then((r:any)=> setDomains(r.data||r||[])).catch(()=>{}); },[]);
  const loadPermsAll = React.useCallback(()=>{ apiFetch("/permissions").then((r:any)=> setPermsAll(r.data||[])).catch(()=>{}); },[]);

  const loadUsers = React.useCallback(async()=>{
    const params = new URLSearchParams({ page:String(usersPage), per_page:String(perPage), search:usersSearch, order_by:usersOrderBy, order:usersOrder });
    if(usersShowAll) params.set("show","all");
    const r = await apiFetch(`/users?${params.toString()}`).catch(()=>({data:[], total:0}));
    setUsers(r.data||[]); setUsersTotal(r.total||0);
  },[usersPage,usersSearch,usersShowAll,usersOrderBy,usersOrder]);
  const loadGroups = React.useCallback(async()=>{
    const params = new URLSearchParams({ page:String(groupsPage), per_page:String(perPage), search:groupsSearch, order_by:groupsOrderBy, order:groupsOrder });
    if(groupsShowAll) params.set("show","all");
    const r = await apiFetch(`/groups?${params.toString()}`).catch(()=>({data:[], total:0}));
    setGroups(r.data||[]); setGroupsTotal(r.total||0);
  },[groupsPage,groupsSearch,groupsShowAll,groupsOrderBy,groupsOrder]);

  React.useEffect(()=>{ loadMe(); loadDomains(); loadPermsAll(); },[loadMe,loadDomains,loadPermsAll]);
  React.useEffect(()=>{ loadUsers(); },[loadUsers]);
  React.useEffect(()=>{ loadGroups(); },[loadGroups]);

  // fetch contacts for dropdown: useEffect fetch /contacts?search= on open
  React.useEffect(()=>{
    if (!userOpen) return;
    const ctrl = new AbortController();
    const q = contactSearch ? `?search=${encodeURIComponent(contactSearch)}&per_page=50` : `?per_page=50`;
    apiFetch(`/contacts${q}`).then((r:any)=> {
      if (ctrl.signal.aborted) return;
      setContacts(r.data||r||[]);
    }).catch(()=> setContacts([]));
    return ()=> ctrl.abort();
  },[userOpen, contactSearch]);

  // delegation: fetch my_level and my_groups for checkbox filtering group_level < my_level || my_groups includes
  React.useEffect(()=>{
    if (!me?.user_uuid) return;
    // derive myLevel from groups that user is member of
    apiFetch(`/users/${me.user_uuid}/groups`).then((r:any)=>{
      const gs = r.data||r||[];
      const maxLvl = Math.max(0, ...gs.map((g:any)=> g.group_level ?? 10));
      setMyLevel(maxLvl || 80);
      setMyGroups(new Set(gs.map((g:any)=> g.group_uuid)));
    }).catch(()=>{
      // fallback: if endpoint fails, assume superadmin level 80
      setMyLevel(80);
      setMyGroups(new Set());
    });
    // also compute level via auth/me if available
    if (me.group_level) setMyLevel(Number(me.group_level));
  },[me]);

  const toggleSort = (target:"users"|"groups", col:string)=>{
    if(target==="users"){
      if(usersOrderBy===col) setUsersOrder(o=>o==="asc"?"desc":"asc"); else { setUsersOrderBy(col); setUsersOrder("asc"); }
    } else {
      if(groupsOrderBy===col) setGroupsOrder(o=>o==="asc"?"desc":"asc"); else { setGroupsOrderBy(col); setGroupsOrder("asc"); }
    }
  };

  // bulk helpers
  const allUsersChecked = users.length>0 && users.every(u=>usersSelected.has(u.user_uuid));
  const allGroupsChecked = groups.length>0 && groups.every(g=>groupsSelected.has(g.group_uuid));
  const bulkUsers = async(action:string)=>{
    const ids = Array.from(usersSelected);
    if(ids.length===0) return;
    if(!confirm(`${action} ${ids.length} users?`)) return;
    try { await apiFetch(`/users/bulk`, {method:"POST", body:JSON.stringify({action, ids})}); setUsersSelected(new Set()); loadUsers(); } catch(e:any){ alert(e.message); }
  };
  const bulkGroups = async(action:string)=>{
    const ids = Array.from(groupsSelected);
    if(ids.length===0) return;
    if(!confirm(`${action} ${ids.length} groups?`)) return;
    try { await apiFetch(`/groups/bulk`, {method:"POST", body:JSON.stringify({action, ids})}); setGroupsSelected(new Set()); loadGroups(); } catch(e:any){ alert(e.message); }
  };

  // user CRUD parity with user_edit.php
  const startCreateUser = ()=>{
    const dom = me?.domain_id || domains[0]?.domain_id || "";
    setEditingUser(null);
    setUserForm({ username:"", email:"", password:"", password_confirm:"", enabled:true, user_status:"", user_type:"default", contact_uuid:"", user_language:"", user_time_zone:"", api_key:"", user_totp_secret:"", group_uuid_name:"", domain_id:dom });
    setSelectedGroups(new Set());
    setContactSearch("");
    setShowApiKey(false);
    setUserOpen(true);
  };
  const startEditUser = async(u:any)=>{
    setEditingUser(u);
    let full:any = u;
    try { full = await apiFetch(`/users/${u.user_uuid}`); } catch {}
    setUserForm({
      username: full.username||"", email: full.email||"", password:"", password_confirm:"", enabled: !!full.enabled,
      user_status: full.user_status||"", user_type: full.user_type||"default",
      contact_uuid: full.contact_uuid||"", user_language: full.user_language||"", user_time_zone: full.user_time_zone||"",
      api_key: full.api_key||"", user_totp_secret: full.user_totp_secret||"", group_uuid_name:"", domain_id: full.domain_id|| full.domain_uuid|| ""
    });
    // populate selectedGroups from full.groups
    try {
      const gs = full.groups || [];
      setSelectedGroups(new Set(gs.map((g:any)=> g.group_uuid)));
    } catch { setSelectedGroups(new Set()); }
    setContactSearch("");
    setShowApiKey(false);
    setUserOpen(true);
  };
  const createOrUpdateUser = async()=>{
    if (isPasswordWeak) {
      alert(passwordRequired ? "password required" : `weak password: ${pwErrors.join(", ")}`);
      return;
    }
    try{
      if(editingUser){
        const payload:any = {
          username: userForm.username, email: userForm.email, enabled: userForm.enabled,
          user_status: userForm.user_status||undefined, user_type: userForm.user_type,
          contact_uuid: userForm.contact_uuid||undefined,
          user_language: userForm.user_language||undefined, user_time_zone: userForm.user_time_zone||undefined,
          api_key: userForm.api_key||undefined, user_totp_secret: userForm.user_totp_secret||undefined,
        };
        if(userForm.password) { payload.password = userForm.password; payload.password_confirm = userForm.password_confirm; }
        // multi-group checkbox list: send groups array filtered by delegation
        if(selectedGroups.size>0) {
          payload.groups = Array.from(selectedGroups);
          // keep legacy single for compat
          payload.group_uuid_name = Array.from(selectedGroups)[0]+`|`+ (groups.find(g=> g.group_uuid===Array.from(selectedGroups)[0])?.name||"");
        }
        if(userForm.domain_id) payload.domain_uuid = userForm.domain_id;
        await apiFetch(`/users/${editingUser.user_uuid}`, {method:"PATCH", body:JSON.stringify(payload)});
      } else {
        const payload:any = {
          username: userForm.username, email: userForm.email, password: userForm.password, password_confirm: userForm.password_confirm,
          enabled: userForm.enabled, user_status: userForm.user_status, user_type: userForm.user_type,
          contact_uuid: userForm.contact_uuid || undefined,
          user_language: userForm.user_language||undefined, user_time_zone: userForm.user_time_zone||undefined,
          api_key: userForm.api_key||undefined, user_totp_secret: userForm.user_totp_secret||undefined,
          domain_uuid: userForm.domain_id || undefined
        };
        if(!payload.password) payload.password = "TempPass123!";
        // multi-group checkboxes
        if(selectedGroups.size>0) {
          payload.groups = Array.from(selectedGroups);
          payload.group_uuids = Array.from(selectedGroups);
        } else if(userForm.group_uuid_name) {
          payload.group_uuid_name = userForm.group_uuid_name;
        }
        await apiFetch("/users", {method:"POST", body:JSON.stringify(payload)});
      }
      setUserOpen(false); setEditingUser(null); loadUsers(); loadGroups();
    } catch(e:any){ alert(e.message); }
  };
  const deleteUserSingle = async(u:any)=>{
    if(!confirm(`Delete user ${u.username}?`)) return;
    try{ await apiFetch(`/users/${u.user_uuid}`, {method:"DELETE"}); loadUsers(); } catch(e:any){ alert(e.message); }
  };

  // group CRUD parity with group_edit.php
  const startCreateGroup = ()=>{
    const dom = me?.domain_id || "";
    setEditingGroup(null);
    setGroupForm({ name:"", description:"", group_level:30, group_protected:false, domain_id:dom });
    setGroupOpen(true);
  };
  const startEditGroup = async(g:any)=>{
    setEditingGroup(g);
    let full:any=g;
    try{ full = await apiFetch(`/groups/${g.group_uuid}`);} catch{}
    setGroupForm({ name: full.name||"", description: full.description||"", group_level: full.group_level ?? 30, group_protected: !!full.group_protected, domain_id: full.domain_id||"" });
    setGroupOpen(true);
  };
  const createOrUpdateGroup = async()=>{
    try{
      if(editingGroup){
        const payload:any = { name: groupForm.name, description: groupForm.description, group_level: Number(groupForm.group_level), group_protected: groupForm.group_protected };
        if(groupForm.domain_id) payload.domain_id = groupForm.domain_id; else if(groupForm.domain_id==="") payload.domain_id = null;
        await apiFetch(`/groups/${editingGroup.group_uuid}`, {method:"PATCH", body:JSON.stringify(payload)});
      } else {
        const payload:any = { name: groupForm.name, description: groupForm.description, group_level: Number(groupForm.group_level), group_protected: groupForm.group_protected, domain_id: groupForm.domain_id || undefined };
        if(groupForm.domain_id==="") payload.domain_id = null;
        await apiFetch("/groups", {method:"POST", body:JSON.stringify(payload)});
      }
      setGroupOpen(false); setEditingGroup(null); loadGroups();
    } catch(e:any){ alert(e.message); }
  };
  const copyGroupSingle = async(g:any)=>{
    try{ await apiFetch(`/groups/${g.group_uuid}/copy`, {method:"POST"}); loadGroups(); } catch(e:any){ alert(e.message); }
  };
  const toggleGroupSingle = async(g:any)=>{
    try{ await apiFetch(`/groups/${g.group_uuid}/toggle`, {method:"POST"}); loadGroups(); } catch(e:any){ alert(e.message); }
  };
  const deleteGroupSingle = async(g:any)=>{
    if(!confirm(`Delete group ${g.name}?`)) return;
    try{ await apiFetch(`/groups/${g.group_uuid}`, {method:"DELETE"}); loadGroups(); } catch(e:any){ alert(e.message); }
  };

  const openMembers = async(g:any)=>{
    setMembersGroup(g); setMembers([]); setAddMemberId(""); setMembersSelected(new Set()); setMembersOpen(true);
    try{ const r = await apiFetch(`/groups/${g.group_uuid}/users${groupsShowAll?"?show=all":""}`); setMembers(r.data||[]);} catch{ setMembers([]);}
  };
  const addMember = async()=>{
    if(!addMemberId || !membersGroup) return;
    try{ await apiFetch(`/groups/${membersGroup.group_uuid}/users`, {method:"POST", body:JSON.stringify({user_uuid: addMemberId})}); const r = await apiFetch(`/groups/${membersGroup.group_uuid}/users`); setMembers(r.data||[]); loadGroups(); setAddMemberId(""); } catch(e:any){ alert(e.message); }
  };
  const removeMember = async(uid:string)=>{
    if(!membersGroup) return;
    try{ await apiFetch(`/groups/${membersGroup.group_uuid}/users/${uid}`, {method:"DELETE"}); const r = await apiFetch(`/groups/${membersGroup.group_uuid}/users`); setMembers(r.data||[]); loadGroups(); } catch(e:any){ alert(e.message); }
  };
  const bulkRemoveMembers = async()=>{
    // members dialog bulk remove uses checkbox_all — legacy handles checkbox_all flag
    const ids = Array.from(membersSelected);
    const checkbox_all = members.length>0 && members.every((m:any)=> membersSelected.has(m.user_uuid));
    if(!ids.length || !membersGroup) return;
    if(!confirm(`Remove ${ids.length} members?`)) return;
    // if checkbox_all true, backend bulk path could be used, but we loop for parity
    for(const uid of ids){ try{ await apiFetch(`/groups/${membersGroup.group_uuid}/users/${uid}`, {method:"DELETE"});} catch{} }
    // also support bulk via group_member bulk endpoint with checkbox_all payload compat
    if (checkbox_all) {
      try { await apiFetch(`/groups/${membersGroup.group_uuid}/users/bulk`, {method:"POST", body:JSON.stringify({action:"delete", ids, checkbox_all:true})}); } catch {}
    }
    const r = await apiFetch(`/groups/${membersGroup.group_uuid}/users`).catch(()=>({data:[]})); setMembers(r.data||[]); setMembersSelected(new Set()); loadGroups();
  };

  const openPerms = async(g:any)=>{
    setPermsGroup(g); setPermsList([]); setPermsSearch(""); setPermsView(""); setPermsOpen(true);
    try{
      const r = await apiFetch(`/groups/${g.group_uuid}/permissions`);
      setPermsList(r.data||[]);
    } catch{ setPermsList([]); }
  };
  const loadPermsFiltered = async()=>{
    if(!permsGroup) return;
    const params = new URLSearchParams();
    if(permsSearch) params.set("search", permsSearch);
    if(permsView) params.set("view", permsView);
    const r = await apiFetch(`/groups/${permsGroup.group_uuid}/permissions?${params.toString()}`).catch(()=>({data:[]}));
    setPermsList(r.data||[]);
  };
  React.useEffect(()=>{ if(permsOpen) loadPermsFiltered(); }, [permsSearch, permsView]);
  const togglePerm = (idx:number)=>{
    const copy=[...permsList];
    copy[idx].assigned = !copy[idx].assigned;
    setPermsList(copy);
  };
  const togglePermProtected = (idx:number)=>{
    const copy=[...permsList];
    copy[idx].protected = !copy[idx].protected;
    setPermsList(copy);
  };
  const savePerms = async()=>{
    if(!permsGroup) return;
    const assigned = permsList.filter(p=>p.assigned).map(p=>p.permission_uuid);
    // perms dialog send protected flag in save: PUT body includes permission_protected
    const permission_protected: Record<string, boolean> = {};
    const protected_map: Record<string, boolean> = {};
    const items = permsList.map((p:any)=> ({
      permission_uuid: p.permission_uuid,
      permission_name: p.name,
      name: p.name,
      assigned: !!p.assigned,
      protected: !!p.protected,
    }));
    permsList.forEach((p:any)=>{
      if(p.protected){
        permission_protected[p.name] = true;
        protected_map[p.name] = true;
        if(p.permission_uuid) protected_map[p.permission_uuid] = true;
      }
    });
    const permission_names = permsList.filter((p:any)=> p.assigned).map((p:any)=> p.name);
    try{
      await apiFetch(`/groups/${permsGroup.group_uuid}/permissions`, {method:"PUT", body:JSON.stringify({permission_uuids: assigned, permission_names, permission_protected, protected_map, items})});
      alert("Permissions saved");
      setPermsOpen(false); loadGroups();
    } catch(e:any){ alert(e.message); }
  };

  // timezones / languages for user edit
  const tzList = React.useMemo(()=>{ try{ return (Intl as any).supportedValuesOf ? (Intl as any).supportedValuesOf("timeZone") : ["UTC","America/New_York","America/Chicago","America/Denver","America/Los_Angeles","Europe/London","Europe/Paris","Asia/Tokyo"]; } catch{ return ["UTC"]; } },[]);
  const langOptions = [
    {code:"en-us", label:"English (US) [en-us]"},
    {code:"en-gb", label:"English (GB) [en-gb]"},
    {code:"de-de", label:"Deutsch [de-de]"},
    {code:"fr-fr", label:"Français [fr-fr]"},
    {code:"es-cl", label:"Español [es-cl]"},
    {code:"pt-br", label:"Português (BR) [pt-br]"},
  ];

  // filtered groups for delegation: group_level < my_level || my_groups includes
  const delegatableGroups = React.useMemo(()=>{
    return groups.filter((g:any)=> (g.group_level ?? 10) < myLevel || myGroups.has(g.group_uuid));
  },[groups, myLevel, myGroups]);

  return (
    <div className="p-4 space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-2xl font-bold">Users & Groups <span className="text-sm font-normal text-muted-foreground">parity with ohngea/core/groups + core/users</span></h1>
        <div className="text-xs font-mono text-muted-foreground hidden md:block">{me ? `${me.username} — ${me.user_uuid} — ${me.domain_id}` : "not logged"}</div>
      </div>

      {/* GROUPS CARD like groups.php */}
      <Card>
        <CardHeader>
          <div className="flex flex-wrap justify-between items-center gap-2">
            <CardTitle className="flex items-center gap-2"><Shield className="h-5 w-5"/> Groups <Badge variant="secondary">{groupsTotal} total</Badge></CardTitle>
            <div className="flex flex-wrap gap-2 items-center">
              <Button variant="outline" size="sm" onClick={()=> startCreateGroup()}><Plus className="h-4 w-4 mr-1"/>Add</Button>
              {groupsSelected.size>0 && <>
                <Button variant="outline" size="sm" onClick={()=>bulkGroups("copy")}><Copy className="h-4 w-4 mr-1"/>Copy ({groupsSelected.size})</Button>
                <Button variant="outline" size="sm" onClick={()=>bulkGroups("toggle")}>Toggle ({groupsSelected.size})</Button>
                <Button variant="destructive" size="sm" onClick={()=>bulkGroups("delete")}><Trash2 className="h-4 w-4 mr-1"/>Delete ({groupsSelected.size})</Button>
              </>}
              <label className="flex items-center gap-1 text-sm border rounded px-2 py-1"><input type="checkbox" checked={groupsShowAll} onChange={e=>{setGroupsShowAll(e.target.checked); setGroupsPage(1);}}/> Show All</label>
              <div className="flex items-center border rounded px-2 py-1 gap-1"><Search className="h-4 w-4"/><Input className="border-0 h-7 w-32 p-1" placeholder="Search" value={groupsSearch} onChange={e=>{setGroupsSearch(e.target.value); setGroupsPage(1);}} /></div>
            </div>
          </div>
          <div className="text-xs text-muted-foreground">Columns: Name, Tools (Permissions / Members), Level, Protected, Description — bulk copy/toggle/delete, Show All, search, order, paging — like groups.php:230 per_page 50</div>
        </CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-8"><input type="checkbox" checked={allGroupsChecked} onChange={e=>{ if(e.target.checked) setGroupsSelected(new Set(groups.map(g=>g.group_uuid))); else setGroupsSelected(new Set()); }}/></TableHead>
                  {groupsShowAll && <TableHead className="cursor-pointer" onClick={()=>toggleSort("groups","domain_name")}>Domain {groupsOrderBy==="domain_name" && (groupsOrder==="asc"?"▲":"▼")}</TableHead>}
                  <TableHead className="cursor-pointer" onClick={()=>toggleSort("groups","group_name")}>Group Name {groupsOrderBy==="group_name" && (groupsOrder==="asc"?"▲":"▼")}</TableHead>
                  <TableHead>Tools</TableHead>
                  <TableHead className="cursor-pointer text-center" onClick={()=>toggleSort("groups","group_level")}>Level {groupsOrderBy==="group_level" && (groupsOrder==="asc"?"▲":"▼")}</TableHead>
                  <TableHead className="cursor-pointer text-center" onClick={()=>toggleSort("groups","group_protected")}>Protected {groupsOrderBy==="group_protected" && (groupsOrder==="asc"?"▲":"▼")}</TableHead>
                  <TableHead>Description</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {groups.map(g=>(
                  <TableRow key={g.group_uuid} className={groupsSelected.has(g.group_uuid)?"bg-muted/50":""}>
                    <TableCell><input type="checkbox" checked={groupsSelected.has(g.group_uuid)} onChange={e=>{ const s=new Set(groupsSelected); if(e.target.checked) s.add(g.group_uuid); else s.delete(g.group_uuid); setGroupsSelected(s); }}/></TableCell>
                    {groupsShowAll && <TableCell className="text-xs">{g.domain_name|| (g.domain_id?"...":"global")}</TableCell>}
                    <TableCell className="font-mono font-bold"><a className="hover:underline cursor-pointer" onClick={()=> startEditGroup(g)}>{g.name}</a></TableCell>
                    <TableCell className="text-xs whitespace-nowrap">
                      <Button variant="link" size="sm" className="h-6 px-1" onClick={()=> openPerms(g)}><Key className="h-3 w-3 mr-1"/>Permissions ({g.group_permissions ?? g.permission_count ?? 0})</Button>
                      <Button variant="link" size="sm" className="h-6 px-1" onClick={()=> openMembers(g)}><UsersIcon className="h-3 w-3 mr-1"/>Members ({g.group_members ?? g.member_count ?? 0})</Button>
                    </TableCell>
                    <TableCell className="text-center">{g.group_level ?? 10}</TableCell>
                    <TableCell className="text-center"><Button variant="ghost" size="sm" className="h-6" onClick={()=> toggleGroupSingle(g)}>{g.group_protected ? "true" : "false"}</Button></TableCell>
                    <TableCell className="text-sm text-muted-foreground max-w-[200px] truncate">{g.description||"-"}</TableCell>
                    <TableCell>
                      <div className="flex gap-1">
                        <Button variant="ghost" size="sm" className="h-7 w-7 p-0" title="Edit" onClick={()=> startEditGroup(g)}><Pencil className="h-4 w-4"/></Button>
                        <Button variant="ghost" size="sm" className="h-7 w-7 p-0" title="Copy" onClick={()=> copyGroupSingle(g)}><Copy className="h-4 w-4"/></Button>
                        <Button variant="ghost" size="sm" className="h-7 w-7 p-0 text-destructive" title="Delete" onClick={()=> deleteGroupSingle(g)}><Trash2 className="h-4 w-4"/></Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
                {groups.length===0 && <TableRow><TableCell colSpan={8} className="text-center text-muted-foreground py-8">No groups — create via Add</TableCell></TableRow>}
              </TableBody>
            </Table>
            <div className="flex justify-between items-center mt-2 text-sm">
              <div>{groupsTotal} rows</div>
              <div className="flex gap-2">
                <Button variant="outline" size="sm" disabled={groupsPage<=1} onClick={()=> setGroupsPage(p=>Math.max(1,p-1))}>Prev</Button>
                <span className="py-1">Page {groupsPage}</span>
                <Button variant="outline" size="sm" disabled={groupsPage*perPage >= groupsTotal} onClick={()=> setGroupsPage(p=>p+1)}>Next</Button>
              </div>
            </div>
          </div>
          {/* mobile groups */}
          <div className="md:hidden grid gap-2">
            {groups.map(g=>(
              <Card key={g.group_uuid} className="p-3">
                <div className="flex justify-between"><div className="font-bold flex items-center gap-2"><input type="checkbox" checked={groupsSelected.has(g.group_uuid)} onChange={e=>{ const s=new Set(groupsSelected); if(e.target.checked) s.add(g.group_uuid); else s.delete(g.group_uuid); setGroupsSelected(s); }}/> <Shield className="h-4 w-4"/>{g.name} <Badge variant="secondary">{g.group_level}</Badge></div><Badge variant={g.group_protected?"default":"outline"}>{g.group_protected?"protected":""}</Badge></div>
                <div className="text-xs text-muted-foreground">{g.description}</div>
                <div className="flex gap-1 mt-2 flex-wrap">
                  <Button variant="outline" size="sm" onClick={()=> openPerms(g)}>Perms ({g.group_permissions??0})</Button>
                  <Button variant="outline" size="sm" onClick={()=> openMembers(g)}>Members ({g.group_members??0})</Button>
                  <Button variant="ghost" size="sm" onClick={()=> startEditGroup(g)}><Pencil className="h-4 w-4"/></Button>
                  <Button variant="ghost" size="sm" onClick={()=> copyGroupSingle(g)}><Copy className="h-4 w-4"/></Button>
                  <Button variant="ghost" size="sm" onClick={()=> deleteGroupSingle(g)}><Trash2 className="h-4 w-4"/></Button>
                </div>
              </Card>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* USERS CARD like users.php:145 per_page 50 */}
      <Card>
        <CardHeader>
          <div className="flex flex-wrap justify-between items-center gap-2">
            <CardTitle className="flex items-center gap-2"><UsersIcon className="h-5 w-5"/> Users <Badge variant="secondary">{usersTotal} total</Badge></CardTitle>
            <div className="flex flex-wrap gap-2 items-center">
              <Button size="sm" onClick={()=> startCreateUser()}><Plus className="h-4 w-4 mr-1"/>Add</Button>
              {usersSelected.size>0 && <>
                <Button variant="outline" size="sm" onClick={()=> bulkUsers("copy")}><Copy className="h-4 w-4 mr-1"/>Copy ({usersSelected.size})</Button>
                <Button variant="outline" size="sm" onClick={()=> bulkUsers("toggle")}>Toggle ({usersSelected.size})</Button>
                <Button variant="destructive" size="sm" onClick={()=> bulkUsers("delete")}><Trash2 className="h-4 w-4 mr-1"/>Delete ({usersSelected.size})</Button>
              </>}
              <label className="flex items-center gap-1 text-sm border rounded px-2 py-1"><input type="checkbox" checked={usersShowAll} onChange={e=>{setUsersShowAll(e.target.checked); setUsersPage(1);}}/> Show All</label>
              <div className="flex items-center border rounded px-2 py-1 gap-1"><Search className="h-4 w-4"/><Input className="border-0 h-7 w-32 p-1" placeholder="Search" value={usersSearch} onChange={e=>{setUsersSearch(e.target.value); setUsersPage(1);}} /></div>
            </div>
          </div>
          <div className="text-xs text-muted-foreground">Columns: Username, Groups, Organization, Name, Note, Enabled — bulk copy/toggle/delete, limit, paging per_page 50, Show All — like users.php:254</div>
        </CardHeader>
        <CardContent>
          <div className="hidden md:block">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-8"><input type="checkbox" checked={allUsersChecked} onChange={e=>{ if(e.target.checked) setUsersSelected(new Set(users.map(u=>u.user_uuid))); else setUsersSelected(new Set()); }}/></TableHead>
                  {usersShowAll && <TableHead className="cursor-pointer" onClick={()=> toggleSort("users","domain_name")}>Domain</TableHead>}
                  <TableHead className="cursor-pointer" onClick={()=> toggleSort("users","username")}>Username {usersOrderBy==="username" && (usersOrder==="asc"?"▲":"▼")}</TableHead>
                  <TableHead>Groups</TableHead>
                  <TableHead>Organization</TableHead>
                  <TableHead>Name</TableHead>
                  <TableHead>Note</TableHead>
                  <TableHead className="cursor-pointer text-center" onClick={()=> toggleSort("users","user_enabled")}>Enabled</TableHead>
                  <TableHead>Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {users.map(u=>(
                  <TableRow key={u.user_uuid} className={usersSelected.has(u.user_uuid)?"bg-muted/50":""}>
                    <TableCell><input type="checkbox" checked={usersSelected.has(u.user_uuid)} onChange={e=>{ const s=new Set(usersSelected); if(e.target.checked) s.add(u.user_uuid); else s.delete(u.user_uuid); setUsersSelected(s); }}/></TableCell>
                    {usersShowAll && <TableCell>{u.domain_name||"-"}</TableCell>}
                    <TableCell className="font-mono"><a className="hover:underline cursor-pointer" onClick={()=> startEditUser(u)}>{u.username}</a></TableCell>
                    <TableCell className="text-xs max-w-[150px] truncate">{u.group_names||"-"}</TableCell>
                    <TableCell className="text-xs">{u.contact_organization||"-"}</TableCell>
                    <TableCell className="text-xs">{u.contact_name||"-"}</TableCell>
                    <TableCell className="text-xs max-w-[150px] truncate">{u.contact_note||u.email||"-"}</TableCell>
                    <TableCell className="text-center"><Button variant="ghost" size="sm" className="h-6" onClick={async()=>{ try{ await apiFetch(`/users/${u.user_uuid}`, {method:"PATCH", body:JSON.stringify({enabled: !u.enabled})}); loadUsers(); } catch(e:any){ alert(e.message); } }}>{u.enabled?"true":"false"}</Button></TableCell>
                    <TableCell><div className="flex gap-1"><Button variant="ghost" className="h-7 w-7 p-0" onClick={()=> startEditUser(u)}><Pencil className="h-4 w-4"/></Button><Button variant="ghost" className="h-7 w-7 p-0 text-destructive" onClick={()=> deleteUserSingle(u)}><Trash2 className="h-4 w-4"/></Button></div></TableCell>
                  </TableRow>
                ))}
                {users.length===0 && <TableRow><TableCell colSpan={8} className="text-center text-muted-foreground py-8">No users</TableCell></TableRow>}
              </TableBody>
            </Table>
            <div className="flex justify-between items-center mt-2 text-sm">
              <div>{usersTotal} rows</div>
              <div className="flex gap-2">
                <Button variant="outline" size="sm" disabled={usersPage<=1} onClick={()=> setUsersPage(p=>Math.max(1,p-1))}>Prev</Button>
                <span className="py-1">Page {usersPage}</span>
                <Button variant="outline" size="sm" disabled={usersPage*perPage >= usersTotal} onClick={()=> setUsersPage(p=>p+1)}>Next</Button>
              </div>
            </div>
          </div>
          <div className="md:hidden grid gap-2">
            {users.map(u=>(
              <Card key={u.user_uuid} className="p-3">
                <div className="flex justify-between"><div className="font-mono font-bold flex gap-2"><input type="checkbox" checked={usersSelected.has(u.user_uuid)} onChange={e=>{ const s=new Set(usersSelected); if(e.target.checked) s.add(u.user_uuid); else s.delete(u.user_uuid); setUsersSelected(s); }}/>{u.username}</div><Badge>{u.enabled?"enabled":"disabled"}</Badge></div>
                <div className="text-xs">Groups: {u.group_names||"-"}</div><div className="text-xs">Org: {u.contact_organization||"-"} Name: {u.contact_name||"-"}</div>
                <div className="flex gap-2 mt-2"><Button className="flex-1" variant="outline" onClick={()=> startEditUser(u)}><Pencil className="h-4 w-4 mr-1"/>Edit</Button><Button className="flex-1" variant="ghost" onClick={()=> deleteUserSingle(u)}><Trash2 className="h-4 w-4 mr-1"/>Delete</Button></div>
              </Card>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* USER EDIT dialog parity with user_edit.php */}
      <Dialog open={userOpen} onOpenChange={(v)=>{ setUserOpen(v); if(!v) setEditingUser(null); }}>
        <DialogHeader><DialogTitle>{editingUser?"Edit User":"Add User"}</DialogTitle></DialogHeader>
        <div className="space-y-3 max-h-[70vh] overflow-auto p-1">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
            <div><Label>Domain *</Label><select value={userForm.domain_id} onChange={e=> setUserForm({...userForm, domain_id:e.target.value})} className="w-full border rounded px-3 py-2 bg-background"><option value="">global</option>{domains.map((d:any)=><option key={d.domain_id} value={d.domain_id}>{d.domain_name}</option>)}</select></div>
            <div><Label>Username *</Label><Input value={userForm.username} onChange={e=> setUserForm({...userForm, username:e.target.value})} placeholder="alice" required /></div>
            <div>
              <Label>{editingUser?"New Password (leave blank to keep)":"Password *"}</Label>
              <Input type="password" value={userForm.password} onChange={e=> setUserForm({...userForm, password:e.target.value})} required={isCreate} placeholder={isCreate?"min 12 chars, number, lower, upper":""} />
              {userForm.password ? (
                <div className="mt-2 space-y-1">
                  <div className="flex gap-1">
                    {[0,1,2,3].map((i)=> (
                      <div key={i} className={`h-1 flex-1 rounded ${i < passwordScore ? (passwordScore===4 ? "bg-green-500" : passwordScore>=2 ? "bg-yellow-500" : "bg-red-500") : "bg-muted"}`} />
                    ))}
                  </div>
                  {pwErrors.length > 0 ? (
                    <p className="text-xs text-destructive">Weak: missing {pwErrors.join(", ")}</p>
                  ) : (
                    <p className="text-xs text-green-600">Strong password</p>
                  )}
                </div>
              ) : isCreate ? (
                <p className="text-xs text-muted-foreground mt-1">Password required — min 12 chars with number, lowercase, uppercase</p>
              ) : null}
            </div>
            <div><Label>Confirm Password</Label><Input type="password" value={userForm.password_confirm} onChange={e=> setUserForm({...userForm, password_confirm:e.target.value})} /></div>
            <div><Label>Email *</Label><Input value={userForm.email} onChange={e=> setUserForm({...userForm, email:e.target.value})} placeholder="alice@example.com" type="email" /></div>
            <div><Label>Language</Label><select value={userForm.user_language} onChange={e=> setUserForm({...userForm, user_language:e.target.value})} className="w-full border rounded px-3 py-2 bg-background"><option value="">default</option>{langOptions.map(o=> <option key={o.code} value={o.code}>{o.label}</option>)}</select></div>
            <div><Label>Time Zone</Label><select value={userForm.user_time_zone} onChange={e=> setUserForm({...userForm, user_time_zone:e.target.value})} className="w-full border rounded px-3 py-2 bg-background"><option value="">default</option>{tzList.map((z:string)=><option key={z} value={z}>{z}</option>)}</select></div>
            <div><Label>Status</Label><select value={userForm.user_status} onChange={e=> setUserForm({...userForm, user_status:e.target.value})} className="w-full border rounded px-3 py-2 bg-background"><option value="">—</option><option value="Available">Available</option><option value="Available (On Demand)">Available (On Demand)</option><option value="Logged Out">Logged Out</option><option value="On Break">On Break</option><option value="Do Not Disturb">Do Not Disturb</option></select></div>
            <div><Label>User Type</Label><select value={userForm.user_type} onChange={e=> setUserForm({...userForm, user_type:e.target.value})} className="w-full border rounded px-3 py-2 bg-background"><option value="default">default</option><option value="virtual">virtual</option><option value="email">email</option></select></div>
            <div>
              <Label>Contact (optional)</Label>
              <Input placeholder="Search contacts…" value={contactSearch} onChange={e=> setContactSearch(e.target.value)} className="mb-1" />
              <select value={userForm.contact_uuid} onChange={e=> setUserForm({...userForm, contact_uuid:e.target.value})} className="w-full border rounded px-3 py-2 bg-background">
                <option value="">— create new —</option>
                {contacts.map((c:any)=><option key={c.contact_uuid} value={c.contact_uuid}>{c.name} {c.organization?`(${c.organization})`:""} — {c.email||c.number||""}</option>)}
              </select>
              <p className="text-xs text-muted-foreground mt-1">Fetched via GET /contacts?search= on open</p>
            </div>
            <div>
              <Label>API Key</Label>
              <div className="flex gap-2">
                <Input type={showApiKey ? "text" : "password"} value={userForm.api_key} onChange={e=> setUserForm({...userForm, api_key:e.target.value})} placeholder="auto" className="flex-1" />
                <Button type="button" variant="outline" size="sm" onClick={()=> setShowApiKey(v=>!v)}>{showApiKey ? "Hide" : "View"}</Button>
              </div>
            </div>
            <div><Label>TOTP Secret</Label><Input value={userForm.user_totp_secret} onChange={e=> setUserForm({...userForm, user_totp_secret:e.target.value})} placeholder="JBSWY3DPE..."/></div>
            <div className="flex items-center gap-2"><input type="checkbox" checked={userForm.enabled} onChange={e=> setUserForm({...userForm, enabled:e.target.checked})} className="h-4 w-4"/><Label>{userForm.enabled?"Enabled":"Disabled"}</Label></div>
            <div className="md:col-span-2">
              <Label>Assign Groups (delegation: group_level &lt; my_level || my_groups includes)</Label>
              <div className="border rounded p-2 max-h-40 overflow-auto space-y-1 bg-background">
                {delegatableGroups.length===0 && <p className="text-xs text-muted-foreground">No delegatable groups — need lower level or membership</p>}
                {delegatableGroups.map((g:any)=> (
                  <label key={g.group_uuid} className="flex items-center gap-2 text-sm cursor-pointer">
                    <input type="checkbox" checked={selectedGroups.has(g.group_uuid)} onChange={e=>{ const s=new Set(selectedGroups); if(e.target.checked) s.add(g.group_uuid); else s.delete(g.group_uuid); setSelectedGroups(s); }} className="h-4 w-4" />
                    <span className="font-mono">{g.name}</span><span className="text-xs text-muted-foreground">(level {g.group_level}) {g.domain_name?`@ ${g.domain_name}`:""}</span>
                  </label>
                ))}
              </div>
              <p className="text-xs text-muted-foreground mt-1">Filtered by delegation: group_level &lt; my_level ({myLevel}) || my_groups includes</p>
            </div>
          </div>
          <Button onClick={createOrUpdateUser} className="w-full" disabled={!!isPasswordWeak}>{editingUser?"Save":"Create"}</Button>
        </div>
      </Dialog>

      <Dialog open={groupOpen} onOpenChange={(v)=>{ setGroupOpen(v); if(!v) setEditingGroup(null); }}>
        <DialogHeader><DialogTitle>{editingGroup?"Edit Group":"Add Group"}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div><Label>Group Name *</Label><Input value={groupForm.name} onChange={e=> setGroupForm({...groupForm, name:e.target.value})} placeholder="sales"/></div>
          <div><Label>Domain</Label><select value={groupForm.domain_id} onChange={e=> setGroupForm({...groupForm, domain_id:e.target.value})} className="w-full border rounded px-3 py-2 bg-background"><option value="">global</option>{domains.map((d:any)=><option key={d.domain_id} value={d.domain_id}>{d.domain_name}</option>)}</select></div>
          <div><Label>Level (10-90 step 10)</Label><select value={groupForm.group_level} onChange={e=> setGroupForm({...groupForm, group_level:Number(e.target.value)})} className="w-full border rounded px-3 py-2 bg-background">{[10,20,30,40,50,60,70,80,90].map(l=> <option key={l} value={l}>{l}</option>)}</select></div>
          <div className="flex items-center gap-2"><input type="checkbox" checked={groupForm.group_protected} onChange={e=> setGroupForm({...groupForm, group_protected:e.target.checked})} className="h-4 w-4"/><Label>Protected</Label></div>
          <div><Label>Description</Label><Input value={groupForm.description} onChange={e=> setGroupForm({...groupForm, description:e.target.value})} placeholder="Sales team"/></div>
          <Button onClick={createOrUpdateGroup} className="w-full">{editingGroup?"Save":"Create"}</Button>
        </div>
      </Dialog>

      <Dialog open={membersOpen} onOpenChange={(v)=>{ setMembersOpen(v); if(!v) setMembersGroup(null); }}>
        <DialogHeader><DialogTitle>Members of {membersGroup?.name} <Badge>{members.length}</Badge></DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="flex gap-2">
            <select value={addMemberId} onChange={e=> setAddMemberId(e.target.value)} className="flex-1 border rounded px-3 py-2 bg-background">
              <option value="">Select user to add</option>
              {users.filter(u=> !members.some((m:any)=> m.user_uuid===u.user_uuid)).map((u:any)=> <option key={u.user_uuid} value={u.user_uuid}>{u.username} ({u.email||"no email"})</option>)}
            </select>
            <Button onClick={addMember} disabled={!addMemberId}><Plus className="h-4 w-4 mr-1"/>Add</Button>
          </div>
          {membersSelected.size>0 && <Button variant="destructive" size="sm" onClick={bulkRemoveMembers}><Trash2 className="h-4 w-4 mr-1"/>Remove Selected ({membersSelected.size})</Button>}
          <div className="border rounded divide-y max-h-64 overflow-auto">
            <div className="p-2 flex items-center gap-2 border-b"><input type="checkbox" name="checkbox_all" checked={members.length>0 && members.every((m:any)=> membersSelected.has(m.user_uuid))} onChange={e=>{ if(e.target.checked) setMembersSelected(new Set(members.map((m:any)=> m.user_uuid))); else setMembersSelected(new Set()); }}/> Select All (checkbox_all)</div>
            {members.length===0 && <div className="p-4 text-center text-muted-foreground text-sm">No members</div>}
            {members.map((m:any)=>(
              <div key={m.user_uuid} className="flex justify-between items-center p-2">
                <label className="flex items-center gap-2"><input type="checkbox" checked={membersSelected.has(m.user_uuid)} onChange={e=>{ const s=new Set(membersSelected); if(e.target.checked) s.add(m.user_uuid); else s.delete(m.user_uuid); setMembersSelected(s); }}/> <div><div className="font-mono text-sm">{m.username}</div><div className="text-xs text-muted-foreground">{m.email||"-"}</div></div></label>
                <Button variant="ghost" size="sm" className="text-destructive" onClick={()=> removeMember(m.user_uuid)}><Trash2 className="h-4 w-4"/></Button>
              </div>
            ))}
          </div>
        </div>
      </Dialog>

      <Dialog open={permsOpen} onOpenChange={(v)=>{ setPermsOpen(v); if(!v) setPermsGroup(null); }}>
        <DialogHeader><DialogTitle>Permissions for {permsGroup?.name}</DialogTitle></DialogHeader>
        <div className="space-y-3">
          <div className="flex gap-2">
            <select value={permsView} onChange={e=> setPermsView(e.target.value)} className="border rounded px-3 py-2 bg-background"><option value="">All</option><option value="assigned">Assigned</option><option value="unassigned">Unassigned</option><option value="protected">Protected</option></select>
            <div className="flex-1 flex items-center border rounded px-2 gap-1"><Search className="h-4 w-4"/><Input className="border-0 h-7" placeholder="Search" value={permsSearch} onChange={e=> setPermsSearch(e.target.value)} /></div>
            <Button onClick={loadPermsFiltered}>Filter</Button>
          </div>
          <div className="border rounded divide-y max-h-96 overflow-auto">
            {permsList.length===0 && <div className="p-4 text-center text-muted-foreground text-sm">No permissions</div>}
            {permsList.map((p:any, idx:number)=>(
              <div key={p.permission_uuid} className="flex justify-between items-center p-2 gap-2">
                <label className="flex items-center gap-2 flex-1 cursor-pointer"><input type="checkbox" checked={!!p.assigned} onChange={()=> togglePerm(idx)} className="h-4 w-4"/><span className="font-mono text-xs truncate">{p.name}</span></label>
                <label className="flex items-center gap-1 text-xs"><input type="checkbox" checked={!!p.protected} onChange={()=> togglePermProtected(idx)} className="h-3 w-3"/> protected</label>
              </div>
            ))}
          </div>
          <Button onClick={savePerms} className="w-full">Save Permissions</Button>
          <div className="text-xs text-muted-foreground">Grouped by application like group_permissions.php — search, view assigned/unassigned/protected, reload — PUT includes permission_protected</div>
        </div>
      </Dialog>
    </div>
  );
}
