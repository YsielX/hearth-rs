local card={api_version=1,id="LOOT_233",name="Cursed Disciple",text="<b>Deathrattle:</b> Summon a 5/1 Revenant.",set="LOOTAPALOOZA",type="minion",rarity="common",cost=4,attack=5,health=1,keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self,position)ctx:summon_at(ctx:controller(self),"LOOT_233t",position)end
card.tokens={{id="LOOT_233t",name="Cursed Revenant",text="",set="LOOTAPALOOZA",type="minion",collectible=false,cost=4,attack=5,health=1,tags={"undead"}}}
return card
