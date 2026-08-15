local card={api_version=1,id="EX1_076",name="Pint-Sized Summoner",text="The first minion you play each turn costs (1) less.",set="EXPERT1",type="minion",rarity="rare",cost=2,attack=2,health=2}
card.auras={{active_zones={"board"},cost=-1,targets=function(ctx,self)
 if ctx:get_data(self,"used_turn")==ctx:turn()then return{}end
 local r={};for _,e in ipairs(ctx:hand(ctx:controller(self)))do if ctx:entity(e).type=="minion"then r[#r+1]=e end end;return r
end}}
card.triggers={{event="minion_played",timing="after",active_zones={"board"},condition=function(ctx,self,e)return e.entity~=self and e.player==ctx:controller(self)end,effect=function(ctx,self)ctx:set_data(self,"used_turn",ctx:turn())end},{event="card_countered",timing="after",active_zones={"board"},condition=function(ctx,self,e)return e.player==ctx:controller(self)and ctx:entity(e.entity).type=="minion"end,effect=function(ctx,self)ctx:set_data(self,"used_turn",ctx:turn())end}}
return card
