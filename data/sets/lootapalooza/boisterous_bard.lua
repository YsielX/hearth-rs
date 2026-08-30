local card={api_version=1,id="LOOT_152",name="Boisterous Bard",text="<b>Battlecry:</b> Give your other minions +1 Health.",set="LOOTAPALOOZA",type="minion",rarity="common",cost=3,attack=3,health=2,keywords={"battlecry"}}
function card.on_battlecry(ctx,self)for _,e in ipairs(ctx:friendly_minions(self))do if e~=self then cardlib.effects.buff(ctx, e,0,1)end end end
return card
