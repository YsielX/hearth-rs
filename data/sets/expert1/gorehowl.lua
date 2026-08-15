local card={api_version=1,id="EX1_411",name="Gorehowl",text="Attacking a minion costs 1 Attack instead of 1 Durability.",set="EXPERT1",type="weapon",class="warrior",rarity="epic",cost=7,attack=7,health=1}
card.rules={durability_loss=function(ctx,self,current)if ctx:get_data(self,"hit_minion")==1 then return 0 end return current end}
card.triggers={
 {event="attack",timing="before",active_zones={"weapon"},condition=function(ctx,self,e)return e.attacker==ctx:player(ctx:controller(self)).hero end,effect=function(ctx,self,e)ctx:set_data(self,"hit_minion",ctx:entity(e.defender).type=="minion"and 1 or 0)end},
 {event="attack",timing="after",active_zones={"weapon"},condition=function(ctx,self,e)return e.attacker==ctx:player(ctx:controller(self)).hero end,effect=function(ctx,self)if ctx:get_data(self,"hit_minion")==1 then ctx:modify(self,{stat="attack",operation="add",value=-1})end;ctx:set_data(self,"hit_minion",0)end},
}
return card
