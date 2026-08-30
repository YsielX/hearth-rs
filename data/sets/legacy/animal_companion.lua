local card={api_version=1,id="NEW1_031", rarity = "free",name="Animal Companion",text="Summon a random Beast Companion.",set="LEGACY",type="spell",class="hunter",cost=3}
function card.on_play(ctx)ctx:random_value({"NEW1_032","NEW1_033","NEW1_034"},"summon_companion")end
function card.summon_companion(ctx,self,id)ctx:summon(ctx:controller(self),id)end
return card
