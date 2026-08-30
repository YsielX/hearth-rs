local card={api_version=1,id="OG_173",name="Blood of The Ancient One",text="If you control two of these\nat the end of your turn, merge them into 'The Ancient One'.",set="OG",type="minion",rarity="epic",cost=9,attack=9,health=9}
card.triggers={{event="turn_ended",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.player==ctx:controller(self)end,effect=function(ctx,self)ctx:continue_with("merge_bloods")end}}
function card.merge_bloods(ctx,self)local found={};for _,m in ipairs(ctx:friendly_minions(self))do if ctx:entity(m).card_id=="OG_173"then found[#found+1]=m end end;if #found>=2 then local pos=math.min(ctx:board_position(found[1]),ctx:board_position(found[2]));ctx:move(found[1],"removed");ctx:move(found[2],"removed");cardlib.effects.summon_at(ctx, ctx:controller(self),"OG_173a",pos)end end
card.tokens={{id="OG_173a",name="The Ancient One",text="",set="OG",type="minion",cost=9,attack=30,health=30}}
return card
