local basic_powers = {
    "HERO_11bp", "HERO_10bp", "HERO_06bp", "HERO_05bp", "HERO_08bp", "HERO_04bp",
    "HERO_09bp", "HERO_03bp", "HERO_02bp", "HERO_07bp", "HERO_01bp",
}

local card = {
    api_version = 1,
    id = "LOE_076",
    name = "Sir Finley Mrrgglton",
    text = "<b><b>Battlecry:</b> Discover</b> a new basic Hero Power.",
    set = "LOE",
    type = "minion",
    rarity = "legendary",
    cost = 1,
    attack = 1,
    health = 3,
    tags = { "murloc" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local current = ctx:entity(ctx:player(player).hero_power).card_id
    local player_class = ctx:player(player).class
    local pool = {}
    for _, card_id in ipairs(basic_powers) do
        if card_id ~= current and ctx:card_definition(card_id).class ~= player_class then
            pool[#pool + 1] = card_id
        end
    end
    ctx:discover_cards(player, "Choose a new Hero Power", pool, 3, "replace_power")
end

function card.replace_power(ctx, self, card_id)
    ctx:replace_hero_power(ctx:controller(self), card_id)
end

return card
