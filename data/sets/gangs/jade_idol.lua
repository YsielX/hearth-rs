local card = {
    api_version = 1,
    id = "CFM_602",
    name = "Jade Idol",
    text = "<b>Choose One -</b> Summon a{1} {0} <b>Jade Golem</b>; or Shuffle 3 copies of this card into your deck.",
    set = "GANGS",
    type = "spell",
    class = "druid",
    rarity = "rare",
    cost = 1,
    keywords = { "choose_one" },
}

local function summon_jade(ctx, self)
    local player = ctx:controller(self)
    ctx:increment_player_data(player, "jade_golem_count", 1)
    ctx:continue_with("summon_jade_golem")
end

local function shuffle_idols(ctx, self)
    local player = ctx:controller(self)
    for _ = 1, 3 do cardlib.effects.shuffle_card_into_deck(ctx, player, "CFM_602") end
end

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { card_id = "CFM_602a", label = "Summon a Jade Golem" },
        { card_id = "CFM_602b", label = "Shuffle 3 Jade Idols into your deck" },
    }, "chosen")
end

function card.chosen(ctx, self, choice)
    if choice == "CFM_602a" then summon_jade(ctx, self) else shuffle_idols(ctx, self) end
end

function card.on_choose_multiple(ctx, self)
    summon_jade(ctx, self)
    shuffle_idols(ctx, self)
end

function card.summon_jade_golem(ctx, self)
    local player = ctx:controller(self)
    local size = math.min(30, ctx:get_player_data(player, "jade_golem_count"))
    cardlib.effects.summon_with_base_stats(ctx, player, "CFM_712_t01", size, size)
end

card.tokens = {
    { id = "CFM_602a", name = "Cut from Jade", text = "Summon a{1} {0} <b>Jade Golem</b>.", set = "GANGS", type = "spell", class = "druid", collectible = false, cost = 1 },
    { id = "CFM_602b", name = "Jade Stash", text = "Shuffle 3 Jade Idols into your deck.", set = "GANGS", type = "spell", class = "druid", collectible = false, cost = 1 },
}

return card
