local card={api_version=1,id="NEW1_008",name="Ancient of Lore",text="<b>Choose One -</b> Draw 2 cards; or Restore #7 Health.",set="EXPERT1",type="minion",class="druid",rarity="epic",cost=7,attack=7,health=7,keywords={"choose_one"}}
function card.on_choose_one(ctx,self)ctx:choose_options(ctx:controller(self),"Choose One",{{card_id="NEW1_008a",label="Draw 2 cards"},{card_id="NEW1_008b",label="Restore 7 Health"}},"chosen")end
function card.chosen(ctx,self,c)if c=="NEW1_008a" then ctx:draw(ctx:controller(self),2)else ctx:choose_entities(ctx:controller(self),"Choose a character",ctx:characters(),"heal_selected")end end
function card.heal_selected(ctx,self,target)cardlib.effects.heal(ctx, target,7)end
function card.on_choose_multiple(ctx,self)ctx:draw(ctx:controller(self),2);ctx:choose_entities(ctx:controller(self),"Choose a character",ctx:characters(),"heal_selected")end
card.tokens={{id="NEW1_008a",name="Ancient Teachings",text="Draw 2 cards.",set="EXPERT1",type="spell",class="druid",collectible=false,cost=7},{id="NEW1_008b",name="Ancient Secrets",text="Restore 7 Health.",set="EXPERT1",type="spell",class="druid",collectible=false,cost=7}}
return card
