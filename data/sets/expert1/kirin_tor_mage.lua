local function secret(def)for _,k in ipairs(def.keywords or{})do if k=="secret"then return true end end return false end
local card={api_version=1,id="EX1_612",name="Kirin Tor Mage",text="[x]<b>Battlecry:</b> The next <b>Secret</b>\nyou play this turn costs (0).",set="EXPERT1",type="minion",class="mage",rarity="rare",cost=3,attack=4,health=3,keywords={"battlecry"}}
function card.on_battlecry(ctx,self)ctx:set_data(self,"discount",1)end
card.auras={{active_zones={"board","graveyard"},cost_set=0,targets=function(ctx,self)if ctx:get_data(self,"discount")==0 then return{}end;local r={};for _,e in ipairs(ctx:hand(ctx:controller(self)))do if secret(ctx:card_definition(ctx:entity(e).card_id))then r[#r+1]=e end end;return r end}}
card.triggers={
 {event="secret_played",timing="after",active_zones={"board","graveyard"},condition=function(ctx,self,e)return ctx:get_data(self,"discount")==1 and e.player==ctx:controller(self)end,effect=function(ctx,self)ctx:set_data(self,"discount",0)end},
 {event="card_countered",timing="after",active_zones={"board","graveyard"},condition=function(ctx,self,e)return ctx:get_data(self,"discount")==1 and e.player==ctx:controller(self)and secret(ctx:card_definition(ctx:entity(e.entity).card_id))end,effect=function(ctx,self)ctx:set_data(self,"discount",0)end},
 {event="turn_ended",timing="after",active_zones={"board","graveyard"},condition=function(ctx,self,e)return ctx:get_data(self,"discount")==1 and e.player==ctx:controller(self)end,effect=function(ctx,self)ctx:set_data(self,"discount",0)end},
}
return card
