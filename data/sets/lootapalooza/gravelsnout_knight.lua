local card={api_version=1,id="LOOT_154",name="Gravelsnout Knight",text="<b>Battlecry:</b> Summon a random 1-Cost minion for your opponent.",set="LOOTAPALOOZA",type="minion",rarity="rare",cost=1,attack=2,health=3,keywords={"battlecry"}}
function card.on_battlecry(ctx,self)local p={};for _,id in ipairs(ctx:collectible_cards())do local d=ctx:card_definition(id);if d.type=="minion"and d.cost==1 then p[#p+1]=id end end;if #p>0 then ctx:random_value(p,"summon_gravelsnout_minion")end end
function card.summon_gravelsnout_minion(ctx,self,id)ctx:summon(ctx:opponent(ctx:controller(self)),id)end
return card
