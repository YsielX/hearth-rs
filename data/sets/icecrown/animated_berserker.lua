local card={api_version=1,id="ICC_238",name="Animated Berserker",text="After you play a minion, deal 1 damage to it.",set="ICECROWN",type="minion",class="warrior",rarity="common",cost=1,attack=1,health=3}
card.triggers={{event="minion_played",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.player==ctx:controller(self)and event.entity~=self end,effect=function(ctx,self,event)ctx:damage(event.entity,1)end}}
return card
