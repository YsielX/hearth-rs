local card={api_version=1,id="EX1_538",name="Unleash the Hounds",text="For each enemy minion, summon a 1/1 Hound with <b>Charge</b>.",set="EXPERT1",type="spell",class="hunter",rarity="common",cost=3,on_play=function(ctx,self)local p=ctx:controller(self);for _=1,#ctx:enemy_minions(self)do ctx:summon(p,"EX1_538t")end end}
card.tokens={{id="EX1_538t",name="Hound",text="<b>Charge</b>",set="EXPERT1",type="minion",class="hunter",collectible=false,cost=1,attack=1,health=1,tags={"beast"},keywords={"charge"}}}
return card
