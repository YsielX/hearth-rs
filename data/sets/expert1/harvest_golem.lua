local card={api_version=1,id="EX1_556",name="Harvest Golem",text="<b>Deathrattle:</b> Summon a 2/1 Damaged Golem.",set="EXPERT1",type="minion",rarity="common",cost=3,attack=2,health=3,tags={"mech"},keywords={"deathrattle"},on_deathrattle=function(ctx,self,pos)cardlib.effects.summon_at(ctx, ctx:controller(self),"skele21",pos)end}
card.tokens={{id="skele21",name="Damaged Golem",text="",set="EXPERT1",type="minion",collectible=false,cost=1,attack=2,health=1,tags={"mech"}}}
return card
