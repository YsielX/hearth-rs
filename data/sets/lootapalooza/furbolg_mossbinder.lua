local card={api_version=1,id="LOOT_150",name="Furbolg Mossbinder",text="<b>Battlecry:</b> Transform a friendly minion into a 6/6 Elemental.",set="LOOTAPALOOZA",type="minion",rarity="rare",cost=5,attack=1,health=1,keywords={"battlecry"},target_mode="required_if_available",targets=function(ctx,self)local r={};for _,e in ipairs(ctx:friendly_minions(self))do if e~=self then r[#r+1]=e end end;return r end}
function card.on_battlecry(ctx,self,target)if target then ctx:transform(target,"LOOT_150t1")end end
card.tokens={{id="LOOT_150t1",name="Moss Elemental",text="",set="LOOTAPALOOZA",type="minion",collectible=false,cost=6,attack=6,health=6,tags={"elemental"}}}
return card
