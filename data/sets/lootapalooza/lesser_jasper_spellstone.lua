local function stone(id,name,text,damage,next_id)
 local x={api_version=1,id=id,name=name,text=text,set="LOOTAPALOOZA",type="spell",class="druid",collectible=false,rarity="rare",spell_school="nature",cost=1,target_mode="required",targets=function(ctx)return ctx:minions()end,on_play=function(ctx,self,target)ctx:damage(target,damage)end}
 if next_id then x.triggers={{event="armor_gained",timing="after",active_zones={"hand"},condition=function(ctx,self,event)return event.player==ctx:controller(self)end,effect=function(ctx,self,event)local n=(ctx:get_data(self,"jasper_armor")or 0)+event.amount;if n>=3 then ctx:set_data(self,"jasper_armor",n-3);ctx:transform_preserving_scripts(self,next_id)else ctx:set_data(self,"jasper_armor",n)end end}} end
 return x
end
local card=stone("LOOT_051","Lesser Jasper Spellstone","Deal $2 damage to a minion. <i>(Gain 3 Armor to upgrade.)</i>",2,"LOOT_051t1");card.collectible=true
card.tokens={stone("LOOT_051t1","Jasper Spellstone","Deal $4 damage to a minion.",4,"LOOT_051t2"),stone("LOOT_051t2","Greater Jasper Spellstone","Deal $6 damage to a minion.",6,nil)}
return card
