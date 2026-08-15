local card={api_version=1,id="CS2_063",name="Corruption",text="Choose an enemy minion. At the start of your turn, destroy it.",set="LEGACY",type="spell",class="warlock",spell_school="shadow",cost=1,target_mode="required",targets=function(ctx,self)return ctx:enemy_minions(self)end}
function card.on_play(ctx,self,target)ctx:set_data(target,"CS2_063_owner",ctx:controller(self));ctx:attach_script(target,"CS2_063")end
card.triggers={{event="turn_started",timing="after",active_zones={"board"},condition=function(ctx,self,e)return e.player==ctx:get_data(self,"CS2_063_owner")end,effect=function(ctx,self)ctx:destroy(self)end}}
return card
