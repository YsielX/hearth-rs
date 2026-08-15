local card={api_version=1,id="EX1_571",name="Force of Nature",text="Summon three 2/2 Treants.",set="EXPERT1",type="spell",class="druid",rarity="epic",spell_school="nature",cost=5,on_play=function(ctx,self)local p=ctx:controller(self);for _=1,3 do ctx:summon(p,"EX1_tk9")end end}
card.tokens={{id="EX1_tk9",name="Treant",text="",set="EXPERT1",type="minion",class="druid",collectible=false,cost=1,attack=2,health=2}}
return card
