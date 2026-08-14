local card={api_version=1,id="OG_249",name="Infested Tauren",text="<b>Taunt</b>\n<b>Deathrattle:</b> Summon a 2/2 Slime.",set="OG",type="minion",rarity="common",cost=4,attack=2,health=3,keywords={"taunt","deathrattle"}}
function card.on_deathrattle(ctx,self,position)ctx:summon_at(ctx:controller(self),"OG_249a",position)end
card.tokens={{id="OG_249a",name="Slime",text="",set="OG",type="minion",cost=2,attack=2,health=2}}
return card
