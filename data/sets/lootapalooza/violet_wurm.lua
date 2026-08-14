local card={api_version=1,id="LOOT_153",name="Violet Wurm",text="<b>Deathrattle:</b> Summon seven 1/1 Grubs.",set="LOOTAPALOOZA",type="minion",rarity="common",cost=8,attack=7,health=7,tags={"beast"},keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self,position)local p=ctx:controller(self);for i=1,7 do ctx:summon_at(p,"LOOT_153t1",position+i-1)end end
card.tokens={{id="LOOT_153t1",name="Grub",text="",set="LOOTAPALOOZA",type="minion",collectible=false,cost=1,attack=1,health=1,tags={"beast"}}}
return card
