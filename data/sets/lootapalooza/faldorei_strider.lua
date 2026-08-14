local ambush={id="LOOT_026e",name="Spider Ambush!",text="<b>Casts When Drawn</b>\nSummon a 4/4 Spider.",set="LOOTAPALOOZA",type="spell",class="rogue",collectible=false,cost=4,keywords={"casts_when_drawn"}}
function ambush.on_play(ctx,self)ctx:summon(ctx:controller(self),"LOOT_026t")end
local card={api_version=1,id="LOOT_026",name="Fal'dorei Strider",text="[x]<b>Battlecry:</b> Shuffle 3\nAmbushes into your deck.\nWhen drawn, summon\na 4/4 Spider.",set="LOOTAPALOOZA",type="minion",class="rogue",rarity="epic",cost=4,attack=4,health=4,keywords={"battlecry"}}
function card.on_battlecry(ctx,self)local p=ctx:controller(self);ctx:shuffle_card_into_deck(p,"LOOT_026e");ctx:shuffle_card_into_deck(p,"LOOT_026e");ctx:shuffle_card_into_deck(p,"LOOT_026e")end
card.tokens={ambush,{id="LOOT_026t",name="Leyline Spider",text="",set="LOOTAPALOOZA",type="minion",class="rogue",collectible=false,cost=4,attack=4,health=4,tags={"beast"}}}
return card
