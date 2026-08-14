local KEY="next_spell_health_cost"
local function is_spell(ctx,e) return ctx:entity(e).type=="spell" end
local function consume(ctx,self,event) local p=ctx:controller(self);return event.player==p and ctx:get_player_data(p,KEY)>0 and is_spell(ctx,event.entity) end
local function use(ctx,self) local p=ctx:controller(self);local n=math.max(0,ctx:get_player_data(p,KEY)-1);ctx:set_player_data(p,KEY,n);if n==0 then ctx:disable_player_keyword(p,"next_spell_costs_health") end end
return {api_version=1,module_type="keyword",id="next_spell_costs_health",name="Next Spell Costs Health",
 auras={{active_zones={"hero"},keywords={"costs_health_instead_of_mana"},targets=function(ctx,self) local p=ctx:controller(self);local r={} if ctx:get_player_data(p,KEY)>0 then for _,e in ipairs(ctx:hand(p)) do if is_spell(ctx,e) then r[#r+1]=e end end end return r end}},
 triggers={{event="card_played",timing="after",active_zones={"hero"},condition=consume,effect=use},{event="card_countered",timing="after",active_zones={"hero"},condition=consume,effect=use},{event="turn_ended",timing="after",active_zones={"hero"},condition=function(ctx,self,event) return event.player==ctx:controller(self) and ctx:get_player_data(event.player,KEY)>0 end,effect=function(ctx,self,event) ctx:set_player_data(event.player,KEY,0);ctx:disable_player_keyword(event.player,"next_spell_costs_health") end}}}
