local card={api_version=1,id="ICC_902",name="Mindbreaker",text="Hero Powers are disabled.",set="ICECROWN",type="minion",rarity="rare",cost=3,attack=2,health=5,tags={"undead"}}
card.auras={{active_zones={"board"},keywords={"hero_power_disabled"},targets=function(ctx,self)return{ctx:player(0).hero_power,ctx:player(1).hero_power}end}}
return card
