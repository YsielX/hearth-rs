local card={api_version=1,id="ICC_408",name="Val'kyr Soulclaimer",text="[x]After this minion\nsurvives damage,\nsummon a 2/2 Ghoul.",set="ICECROWN",type="minion",class="warrior",rarity="rare",cost=3,attack=1,health=4,tags={"undead"}}
card.triggers={{event="damaged",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.target==self and event.amount>0 and ctx:entity(self).health>0 end,effect=function(ctx,self)ctx:summon(ctx:controller(self),"ICC_900t")end}}
return card
