local card={api_version=1,id="EX1_341",name="Lightwell",text="At the start of your turn, restore #3 Health to a damaged friendly character.",set="EXPERT1",type="minion",class="priest",rarity="rare",cost=2,attack=0,health=5}
card.triggers={{event="turn_started",timing="after",active_zones={"board"},condition=function(ctx,self,e)return e.player==ctx:controller(self)end,effect=function(ctx,self)local r={};local p=ctx:controller(self);for _,e in ipairs(ctx:characters())do local x=ctx:entity(e);if x.controller==p and x.health<x.max_health then r[#r+1]=e end end;if #r>0 then ctx:random_entity(r,"lightwell_target")end end}}
function card.lightwell_target(ctx,self,target)cardlib.effects.heal(ctx, target,3)end
return card
