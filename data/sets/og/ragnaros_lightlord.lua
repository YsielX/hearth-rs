local card={api_version=1,id="OG_229",name="Ragnaros, Lightlord",text="At the end of your turn, restore #8 Health to a damaged friendly character.",set="OG",type="minion",class="paladin",rarity="legendary",cost=8,attack=8,health=8,tags={"elemental"}}
card.triggers={{event="turn_ended",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.player==ctx:controller(self)end,effect=function(ctx,self)local p=ctx:controller(self);local pool={};for _,e in ipairs(ctx:characters())do local x=ctx:entity(e);if x.controller==p and x.damage>0 then pool[#pool+1]=e end end;if #pool>0 then ctx:random_entity(pool,"heal_character")end end}}
function card.heal_character(ctx,self,target)cardlib.effects.heal(ctx, target,8)end
return card
