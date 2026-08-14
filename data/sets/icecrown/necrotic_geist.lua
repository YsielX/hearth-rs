local card={api_version=1,id="ICC_900",name="Necrotic Geist",text="Whenever one of your other minions dies, summon a 2/2 Ghoul.",set="ICECROWN",type="minion",rarity="common",cost=6,attack=5,health=3,tags={"undead"}}
card.triggers={{event="entity_died",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.player==ctx:controller(self)and event.entity~=self end,effect=function(ctx,self)ctx:summon(ctx:controller(self),"ICC_900t")end}}
card.tokens={{id="ICC_900t",name="Ghoul",text="",set="ICECROWN",type="minion",collectible=false,cost=2,attack=2,health=2,tags={"undead"}}}
return card
