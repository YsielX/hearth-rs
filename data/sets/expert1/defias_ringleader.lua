local card={api_version=1,id="EX1_131",name="Defias Ringleader",text="<b>Combo:</b> Summon a 2/1 Defias Bandit.",set="EXPERT1",type="minion",class="rogue",rarity="common",cost=2,attack=3,health=2,keywords={"combo"},on_combo=function(ctx,self)ctx:summon(ctx:controller(self),"EX1_131t")end}
card.tokens={{id="EX1_131t",name="Defias Bandit",text="",set="EXPERT1",type="minion",class="rogue",collectible=false,cost=1,attack=2,health=1,tags={"pirate"}}}
return card
