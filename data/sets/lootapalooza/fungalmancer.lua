local card={api_version=1,id="LOOT_167",name="Fungalmancer",text="<b>Battlecry:</b> Give adjacent minions +2/+2.",set="LOOTAPALOOZA",type="minion",rarity="common",cost=5,attack=2,health=2,keywords={"battlecry"}}
function card.on_battlecry(ctx,self)for _,e in ipairs(ctx:adjacent_minions(self))do ctx:buff(e,2,2)end end
return card
