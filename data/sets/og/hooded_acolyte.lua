local card={api_version=1,id="OG_334",name="Hooded Acolyte",text="[x]<b>Taunt</b>\nWhenever a character is\nhealed, give your C'Thun\n+1/+1 <i>(wherever it is).</i>",set="OG",type="minion",class="priest",rarity="common",cost=4,attack=3,health=6,keywords={"taunt"}}
card.triggers={{event="healed",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.amount>0 end,effect=function(ctx)ctx:continue_with("buff_cthun")end}}
function card.buff_cthun(ctx,self)local p=ctx:controller(self);ctx:grant_player_keyword(p,"cthun_buffs");ctx:increment_player_data(p,"cthun_attack_buff",1);ctx:increment_player_data(p,"cthun_health_buff",1)end
return card
