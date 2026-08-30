local card={api_version=1,id="EX1_110",name="Cairne Bloodhoof",text="[x]<b>Taunt</b>\n<b>Deathrattle:</b> Summon a\n5/5 Baine Bloodhoof.",set="EXPERT1",type="minion",rarity="legendary",cost=6,attack=5,health=5,keywords={"taunt","deathrattle"},on_deathrattle=function(ctx,self,pos)cardlib.effects.summon_at(ctx, ctx:controller(self),"EX1_110t",pos)end}
card.tokens={{id="EX1_110t",name="Baine Bloodhoof",text="",set="EXPERT1",type="minion",collectible=false,cost=5,attack=5,health=5}}
return card
