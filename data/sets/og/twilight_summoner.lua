local card={api_version=1,id="OG_272",name="Twilight Summoner",text="<b>Deathrattle:</b> Summon a 5/5 Faceless Destroyer.",set="OG",type="minion",rarity="epic",cost=4,attack=1,health=1,keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self,position)cardlib.effects.summon_at(ctx, ctx:controller(self),"OG_272t",position)end
card.tokens={{id="OG_272t",name="Faceless Destroyer",text="",set="OG",type="minion",cost=4,attack=5,health=5}}
return card
