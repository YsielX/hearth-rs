local card={api_version=1,id="EX1_534",name="Savannah Highmane",text="<b>Deathrattle:</b> Summon two 2/2 Hyenas.",set="EXPERT1",type="minion",class="hunter",rarity="rare",cost=6,attack=7,health=5,tags={"beast"},keywords={"deathrattle"},on_deathrattle=function(ctx,self,pos)local p=ctx:controller(self);cardlib.effects.summon_at(ctx, p,"EX1_534t",pos);cardlib.effects.summon_at(ctx, p,"EX1_534t",pos)end}
card.tokens={{id="EX1_534t",name="Hyena",text="",set="EXPERT1",type="minion",class="hunter",collectible=false,cost=2,attack=2,health=2,tags={"beast"}}}
return card
