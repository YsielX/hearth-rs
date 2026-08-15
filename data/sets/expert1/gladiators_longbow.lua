local card={api_version=1,id="DS1_188",name="Gladiator's Longbow",text="Your hero is <b>Immune</b> while attacking.",set="EXPERT1",type="weapon",class="hunter",rarity="epic",cost=7,attack=5,health=2}
card.triggers={
 {event="attack",timing="before",active_zones={"weapon"},condition=function(ctx,self,e)return e.attacker==ctx:player(ctx:controller(self)).hero end,effect=function(ctx,self,e)ctx:set_data(self,"attacking",e.event_id)end},
 {event="damaged",timing="before",active_zones={"weapon"},condition=function(ctx,self,e)return ctx:get_data(self,"attacking")~=0 and e.target==ctx:player(ctx:controller(self)).hero end,effect=function(ctx,self,e)ctx:cancel_event(e)end},
 {event="attack",timing="after",active_zones={"weapon"},condition=function(ctx,self,e)return e.attacker==ctx:player(ctx:controller(self)).hero end,effect=function(ctx,self)ctx:set_data(self,"attacking",0)end},
 {event="turn_ended",timing="after",active_zones={"weapon"},condition=function(ctx,self,e)return e.player==ctx:controller(self)end,effect=function(ctx,self)ctx:set_data(self,"attacking",0)end},
}
return card
